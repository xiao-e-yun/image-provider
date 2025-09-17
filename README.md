# Image Provider

Axum-based image resizer. (like [node/ipx](https://www.npmjs.com/package/ipx))

## Features
- Realtime Resizing
- Embeddable
- Caching

## Format
```
http://localhost:3000/path/to/image.jpg?dpr=2&w=100&h=100&output=webp
```
> Such as `wsrv.nl`

Output Format
`output: "webp", "jpeg", "png"`

Device Pixel Ratio
`dpr: 1 ~ 3`

Width
`w: Number`

Height
`h: Number`

## Usage
### Cli
Download binary from [releases](https://github.com/xiao-e-yun/image-provider/releases).
```bash
Usage: image-provider [OPTIONS]

Options:
  -p, --port <PORT>
          [default: 3000]
      --resize-path <PATH>
          [default: .]
      --resize-filter-type <FILTER_TYPE>
          `lanczos3` `gaussian` `catmull-rom` `hamming` `mitchell` `bilinear` `box` [default: lanczos3]
      --resize-algorithm <ALGORITHM>
          Slow <- `super-sampling8x` `super-sampling4x` `super-sampling2x` `convolution` `interpolation` `nearest` -> Fast (nearest will ignore filter_type) [default: interpolation]
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
  -h, --help
          Print help
```

### Programmatic API
Add `image_provider` to your `Cargo.toml`:
```bash
cargo add image_provider
```

Create image provider router:
```rust
use image_provider::{get_images_router, ResizeConfig};

// Create images_provider router
let config = ResizeConfig::builder().build();
let images_router: Router = get_images_router(config);
```

Then you can mount `images_router` to your main router.
