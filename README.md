# rgbcx-sys

Rust FFI bindings to the rgbcx (BC1-BC5 decoder/encoder) from Rich Geldreich's bc7enc_rdo project.
This provides access to BC1-BC5 texture compression and decompression functionality through the underlying C++ library.

I made these bindings for fuzz testing my own implementations only, so they're not super polished;
but feel free to use for anything else.

## Example Usage

```rust
use rgbcx_sys::root::rgbcx;

// Initialize the library
unsafe { rgbcx::init(rgbcx::bc1_approx_mode::cBC1Ideal) };

// Example: Encode and decode a BC1 block
fn encode_decode_example() {
    // Create a test image: 4x4 checkerboard of red and green
    let mut source_pixels = [0u8; 16 * 4]; // RGBA 4x4 block

    // Create a red/green checkerboard pattern
    for y in 0..4 {
        for x in 0..4 {
            let pixel_idx = (y * 4 + x) * 4;
            if (x + y) % 2 == 0 {
                // Red pixel
                source_pixels[pixel_idx] = 255;     // R
                source_pixels[pixel_idx + 1] = 0;   // G
                source_pixels[pixel_idx + 2] = 0;   // B
                source_pixels[pixel_idx + 3] = 255; // A
            } else {
                // Green pixel
                source_pixels[pixel_idx] = 0;       // R
                source_pixels[pixel_idx + 1] = 255; // G
                source_pixels[pixel_idx + 2] = 0;   // B
                source_pixels[pixel_idx + 3] = 255; // A
            }
        }
    }

    // BC1 encoded block (8 bytes)
    let mut encoded_block = [0u8; 8];

    // Encode the source pixels to BC1
    unsafe {
        rgbcx::encode_bc1(
            10, // quality level
            encoded_block.as_mut_ptr() as *mut std::ffi::c_void,
            source_pixels.as_ptr(),
            true, // allow 3-color blocks
            false, // don't use transparent texels for black
            std::ptr::null() // pForce_selectors parameter (optional)
        );
    }

    // Now decode the BC1 block back
    let mut decoded_pixels = [0u8; 16 * 4];
    unsafe {
        rgbcx::unpack_bc1(
            encoded_block.as_ptr() as *const std::ffi::c_void,
            decoded_pixels.as_mut_ptr() as *mut std::ffi::c_void,
            true,
            rgbcx::bc1_approx_mode::cBC1Ideal,
        );
    }
}
```
