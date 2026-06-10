#!/usr/bin/env python3
import argparse
import gzip
import hashlib
import json
import shutil
import tarfile
import tempfile
from pathlib import Path


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def read_tar_member(tar, name):
    member = tar.getmember(name)
    fileobj = tar.extractfile(member)
    if fileobj is None:
        raise RuntimeError(f"tar member is not a file: {name}")
    return fileobj.read()


def copy_metadata_file(src_root, dst_root, name):
    src = src_root / name
    if src.exists():
        shutil.copy2(src, dst_root / name)


def docker_archive_to_oci(docker_archive, images_dir, ref_name):
    images_dir.mkdir(parents=True, exist_ok=True)
    blobs_dir = images_dir / "blobs" / "sha256"
    blobs_dir.mkdir(parents=True, exist_ok=True)
    (images_dir / "oci-layout").write_text(
        json.dumps({"imageLayoutVersion": "1.0.0"}, separators=(",", ":"))
    )

    with tarfile.open(docker_archive, "r:*") as docker_tar:
        manifest_list = json.loads(read_tar_member(docker_tar, "manifest.json"))
        if len(manifest_list) != 1:
            raise RuntimeError(f"expected one image in docker archive, got {len(manifest_list)}")
        docker_manifest = manifest_list[0]

        config_name = docker_manifest["Config"]
        config_bytes = read_tar_member(docker_tar, config_name)
        config_digest = f"sha256:{sha256_bytes(config_bytes)}"
        (blobs_dir / config_digest.removeprefix("sha256:")).write_bytes(config_bytes)

        layers = []
        for layer_name in docker_manifest["Layers"]:
            layer_bytes = read_tar_member(docker_tar, layer_name)
            compressed = gzip.compress(layer_bytes, compresslevel=9, mtime=0)
            digest = f"sha256:{sha256_bytes(compressed)}"
            diff_digest = f"sha256:{sha256_bytes(layer_bytes)}"
            (blobs_dir / digest.removeprefix("sha256:")).write_bytes(compressed)
            layers.append(
                {
                    "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
                    "digest": digest,
                    "size": len(compressed),
                    "annotations": {
                        "org.opencontainers.image.uncompressedDigest": diff_digest
                    },
                }
            )

    oci_manifest = {
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": {
            "mediaType": "application/vnd.oci.image.config.v1+json",
            "digest": config_digest,
            "size": len(config_bytes),
        },
        "layers": layers,
    }
    manifest_bytes = json.dumps(oci_manifest, separators=(",", ":")).encode()
    manifest_digest = f"sha256:{sha256_bytes(manifest_bytes)}"
    (blobs_dir / manifest_digest.removeprefix("sha256:")).write_bytes(manifest_bytes)

    index = {
        "schemaVersion": 2,
        "manifests": [
            {
                "mediaType": "application/vnd.oci.image.manifest.v1+json",
                "digest": manifest_digest,
                "size": len(manifest_bytes),
                "annotations": {"org.opencontainers.image.ref.name": ref_name},
            }
        ],
    }
    (images_dir / "index.json").write_text(json.dumps(index, indent=2) + "\n")

    return config_digest, [layer["digest"] for layer in layers]


def build_lpk(src_root, docker_archive, output, image_ref):
    output = Path(output)
    with tempfile.TemporaryDirectory(prefix="attic-lpk-") as tmp:
        tmpdir = Path(tmp)
        copy_metadata_file(src_root, tmpdir, "package.yml")
        copy_metadata_file(src_root, tmpdir, "icon.png")
        copy_metadata_file(src_root / "assets", tmpdir, "icon.png")

        images_dir = tmpdir / "images"
        config_digest, layer_digests = docker_archive_to_oci(
            docker_archive, images_dir, image_ref
        )

        manifest_template = (src_root / "lzc-manifest.yml").read_text()
        manifest = manifest_template.replace(
            f"embed:{image_ref}", f"embed:{image_ref}@{config_digest}"
        )
        (tmpdir / "manifest.yml").write_text(manifest)

        images_lock = ["version: 1", "images:", f"  {image_ref}:"]
        images_lock.append(f"    image_id: {config_digest}")
        images_lock.append("    upstream: ''")
        images_lock.append("    layers:")
        for digest in layer_digests:
            images_lock.append(f"      - digest: {digest}")
            images_lock.append("        source: embed")
        (tmpdir / "images.lock").write_text("\n".join(images_lock) + "\n")

        output.parent.mkdir(parents=True, exist_ok=True)
        with tarfile.open(output, "w", format=tarfile.PAX_FORMAT) as tar:
            for name in ["icon.png", "images", "images.lock", "manifest.yml", "package.yml"]:
                path = tmpdir / name
                if path.exists():
                    tar.add(path, arcname=name)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--src-root", required=True, type=Path)
    parser.add_argument("--docker-archive", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--image-ref", default="attic")
    args = parser.parse_args()
    build_lpk(args.src_root, args.docker_archive, args.output, args.image_ref)


if __name__ == "__main__":
    main()
