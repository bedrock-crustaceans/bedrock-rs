#!/usr/bin/env python3
"""Dumps one Bedrock world through the Python reference editor's public
load API, to the same normalized line format `xtask/src/oracle/dump.rs`
produces from this crate's decode. Driven by the `diff-oracle` xtask
subcommand; not meant to be edited or vendored, only run.

Usage: python_dump.py <world-dir>

Writes the sorted, deduplicated dump to stdout.
"""

from __future__ import annotations

import sys

import numpy as np
import amulet


def block_key(block) -> str:
    props = sorted(block.properties.items(), key=lambda kv: kv[0])
    key = f"{block.namespace}:{block.base_name}"
    if props:
        key += ";" + ",".join(f"{k}={v!r}" for k, v in props)
    return key


def dump_chunk_blocks(chunk, dim_id: int, cx: int, cz: int, y0: int, y1: int, out: list[str]) -> None:
    palette = chunk.block_palette
    keys = [block_key(palette[i]) for i in range(len(palette))]

    arr = np.asarray(chunk.blocks[:, y0:y1, :])
    xs, ys, zs = np.nonzero(arr >= 0)
    for x, y_off, z in zip(xs.tolist(), ys.tolist(), zs.tolist()):
        idx = int(arr[x, y_off, z])
        if idx == 0 and keys and keys[0].endswith(":air"):
            continue
        y = y0 + y_off
        out.append(f"BLOCK\t{dim_id}\t{cx}\t{cz}\t{x}\t{y}\t{z}\t0\t{keys[idx]}")

    for i, key in enumerate(keys):
        out.append(f"PALETTE\t{dim_id}\t{cx}\t{cz}\t?\t0\t{i}\t{key}")


def dump_chunk_biomes(chunk, dim_id: int, cx: int, cz: int, out: list[str]) -> None:
    palette = chunk.biome_palette
    try:
        arr = np.asarray(chunk.biomes[:, :])
        two_d = True
    except Exception:
        two_d = False
        arr = None

    if two_d and arr is not None and arr.ndim == 2:
        for x in range(arr.shape[0]):
            for z in range(arr.shape[1]):
                idx = int(arr[x, z])
                biome = palette[idx]
                out.append(f"BIOME2D\t{dim_id}\t{cx}\t{cz}\t{x}\t{z}\t{biome}")


def dump_chunk_block_entities(chunk, dim_id: int, cx: int, cz: int, out: list[str]) -> None:
    for (x, y, z), be in chunk.block_entities.items():
        ident = f"{be.namespace}:{be.base_name}"
        out.append(f"BLOCKENTITY\t{dim_id}\t{cx}\t{cz}\t{ident}\t{x}\t{y}\t{z}")


def dump_chunk_entities(chunk, dim_id: int, cx: int, cz: int, out: list[str]) -> None:
    for entity in chunk.entities:
        ident = f"{entity.namespace}:{entity.base_name}"
        out.append(
            f"ENTITY\t{dim_id}\t{cx}\t{cz}\t{ident}\t{entity.x}\t{entity.y}\t{entity.z}"
        )


DIMENSION_IDS = {
    "minecraft:overworld": 0,
    "minecraft:the_nether": 1,
    "minecraft:the_end": 2,
}


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: python_dump.py <world-dir>", file=sys.stderr)
        return 2

    world_dir = sys.argv[1]
    level = amulet.load_level(world_dir)
    out: list[str] = []
    try:
        for dim in level.dimensions:
            dim_id = DIMENSION_IDS.get(dim, -1)
            bounds = level.bounds(dim)
            y0, y1 = 0, 256
            if bounds:
                box = bounds[0]
                y0, y1 = int(box.min_y), int(box.max_y)

            for cx, cz in level.all_chunk_coords(dim):
                try:
                    chunk = level.get_chunk(cx, cz, dim)
                except Exception as err:
                    out.append(f"CHUNKERROR\t{dim_id}\t{cx}\t{cz}\t{err}")
                    continue
                dump_chunk_blocks(chunk, dim_id, cx, cz, y0, y1, out)
                dump_chunk_biomes(chunk, dim_id, cx, cz, out)
                dump_chunk_block_entities(chunk, dim_id, cx, cz, out)
                dump_chunk_entities(chunk, dim_id, cx, cz, out)
    finally:
        level.close()

    out = sorted(set(out))
    sys.stdout.write("\n".join(out))
    if out:
        sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
