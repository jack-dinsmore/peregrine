# Details

Date : 2025-03-29 21:19:18

Directory /Users/jtd/Documents/personal/code/rust/peregrine

Total : 52 files,  4583 codes, 388 comments, 535 blanks, all 5506 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [license.md](/license.md) | Markdown | 4 | 0 | 3 | 7 |
| [peregrine/Cargo.toml](/peregrine/Cargo.toml) | Ignore | 22 | 0 | 3 | 25 |
| [peregrine/build.rs](/peregrine/build.rs) | Rust | 11 | 0 | 3 | 14 |
| [peregrine/src/dev/mod.rs](/peregrine/src/dev/mod.rs) | Rust | 1 | 0 | 0 | 1 |
| [peregrine/src/dev/normal.rs](/peregrine/src/dev/normal.rs) | Rust | 80 | 39 | 12 | 131 |
| [peregrine/src/main.rs](/peregrine/src/main.rs) | Rust | 223 | 61 | 30 | 314 |
| [peregrine/src/shaders/shader\_2d.wgsl](/peregrine/src/shaders/shader_2d.wgsl) | WGSL | 21 | 1 | 5 | 27 |
| [peregrine/src/shaders/shader\_3d.wgsl](/peregrine/src/shaders/shader_3d.wgsl) | WGSL | 79 | 3 | 9 | 91 |
| [peregrine/src/shaders/shader\_placement.wgsl](/peregrine/src/shaders/shader_placement.wgsl) | WGSL | 30 | 2 | 6 | 38 |
| [peregrine/src/shaders/shader\_solid.wgsl](/peregrine/src/shaders/shader_solid.wgsl) | WGSL | 32 | 1 | 5 | 38 |
| [peregrine/src/ship/circuit.rs](/peregrine/src/ship/circuit.rs) | Rust | 20 | 0 | 2 | 22 |
| [peregrine/src/ship/grid.rs](/peregrine/src/ship/grid.rs) | Rust | 170 | 12 | 10 | 192 |
| [peregrine/src/ship/mod.rs](/peregrine/src/ship/mod.rs) | Rust | 143 | 7 | 20 | 170 |
| [peregrine/src/ship/orientation.rs](/peregrine/src/ship/orientation.rs) | Rust | 73 | 2 | 10 | 85 |
| [peregrine/src/ship/panel.rs](/peregrine/src/ship/panel.rs) | Rust | 56 | 1 | 6 | 63 |
| [peregrine/src/ship/part.rs](/peregrine/src/ship/part.rs) | Rust | 245 | 7 | 21 | 273 |
| [peregrine/src/ui/connections.rs](/peregrine/src/ui/connections.rs) | Rust | 102 | 1 | 9 | 112 |
| [peregrine/src/ui/fps.rs](/peregrine/src/ui/fps.rs) | Rust | 38 | 0 | 7 | 45 |
| [peregrine/src/ui/mod.rs](/peregrine/src/ui/mod.rs) | Rust | 26 | 0 | 4 | 30 |
| [peregrine/src/ui/place\_panel.rs](/peregrine/src/ui/place_panel.rs) | Rust | 124 | 8 | 15 | 147 |
| [peregrine/src/ui/place\_part.rs](/peregrine/src/ui/place_part.rs) | Rust | 148 | 13 | 17 | 178 |
| [peregrine/src/util/mod.rs](/peregrine/src/util/mod.rs) | Rust | 2 | 0 | 1 | 3 |
| [peregrine/src/util/perlin.rs](/peregrine/src/util/perlin.rs) | Rust | 0 | 0 | 1 | 1 |
| [peregrine/src/util/save.rs](/peregrine/src/util/save.rs) | Rust | 24 | 0 | 1 | 25 |
| [readme.md](/readme.md) | Markdown | 34 | 0 | 14 | 48 |
| [tethys-proc/Cargo.toml](/tethys-proc/Cargo.toml) | Ignore | 11 | 0 | 3 | 14 |
| [tethys-proc/src/lib.rs](/tethys-proc/src/lib.rs) | Rust | 84 | 0 | 9 | 93 |
| [tethys/Cargo.toml](/tethys/Cargo.toml) | Ignore | 16 | 1 | 2 | 19 |
| [tethys/src/graphics/camera.rs](/tethys/src/graphics/camera.rs) | Rust | 106 | 4 | 16 | 126 |
| [tethys/src/graphics/mod.rs](/tethys/src/graphics/mod.rs) | Rust | 164 | 3 | 21 | 188 |
| [tethys/src/graphics/model/container.rs](/tethys/src/graphics/model/container.rs) | Rust | 122 | 1 | 14 | 137 |
| [tethys/src/graphics/model/loading.rs](/tethys/src/graphics/model/loading.rs) | Rust | 131 | 0 | 19 | 150 |
| [tethys/src/graphics/model/material.rs](/tethys/src/graphics/model/material.rs) | Rust | 138 | 19 | 16 | 173 |
| [tethys/src/graphics/model/mesh.rs](/tethys/src/graphics/model/mesh.rs) | Rust | 58 | 0 | 9 | 67 |
| [tethys/src/graphics/model/mod.rs](/tethys/src/graphics/model/mod.rs) | Rust | 42 | 3 | 11 | 56 |
| [tethys/src/graphics/object.rs](/tethys/src/graphics/object.rs) | Rust | 73 | 20 | 12 | 105 |
| [tethys/src/graphics/primitives.rs](/tethys/src/graphics/primitives.rs) | Rust | 154 | 42 | 19 | 215 |
| [tethys/src/graphics/render\_pass.rs](/tethys/src/graphics/render_pass.rs) | Rust | 75 | 0 | 9 | 84 |
| [tethys/src/graphics/shader.rs](/tethys/src/graphics/shader.rs) | Rust | 199 | 94 | 13 | 306 |
| [tethys/src/io/key.rs](/tethys/src/io/key.rs) | Rust | 287 | 0 | 9 | 296 |
| [tethys/src/io/mod.rs](/tethys/src/io/mod.rs) | Rust | 2 | 0 | 0 | 2 |
| [tethys/src/io/mouse.rs](/tethys/src/io/mouse.rs) | Rust | 25 | 0 | 3 | 28 |
| [tethys/src/lib.rs](/tethys/src/lib.rs) | Rust | 141 | 1 | 15 | 157 |
| [tethys/src/physics/collisions/collision\_box.rs](/tethys/src/physics/collisions/collision_box.rs) | Rust | 192 | 13 | 15 | 220 |
| [tethys/src/physics/collisions/collision\_grid.rs](/tethys/src/physics/collisions/collision_grid.rs) | Rust | 208 | 10 | 20 | 238 |
| [tethys/src/physics/collisions/collision\_line.rs](/tethys/src/physics/collisions/collision_line.rs) | Rust | 27 | 0 | 7 | 34 |
| [tethys/src/physics/collisions/collision\_tree.rs](/tethys/src/physics/collisions/collision_tree.rs) | Rust | 126 | 3 | 11 | 140 |
| [tethys/src/physics/collisions/mod.rs](/tethys/src/physics/collisions/mod.rs) | Rust | 124 | 8 | 24 | 156 |
| [tethys/src/physics/collisions/report.rs](/tethys/src/physics/collisions/report.rs) | Rust | 95 | 4 | 12 | 111 |
| [tethys/src/physics/mod.rs](/tethys/src/physics/mod.rs) | Rust | 100 | 0 | 14 | 114 |
| [tethys/src/util/mod.rs](/tethys/src/util/mod.rs) | Rust | 14 | 2 | 3 | 19 |
| [tethys/src/util/tree.rs](/tethys/src/util/tree.rs) | Rust | 161 | 2 | 15 | 178 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)