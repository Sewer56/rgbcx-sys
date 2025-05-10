#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]
#![allow(warnings)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use crate::root::rgbcx;    

    #[test]
    fn it_works() {
        unsafe { rgbcx::init(rgbcx::bc1_approx_mode::cBC1Ideal) };
    }

    #[test]
    fn can_decode_bc1_block() {
        // Test case: Simple red color
        let bc1_block: [u8; 8] = [
            0x00, 0xF8, // c0 = R:31 G:0 B:0
            0x00, 0xF8, // c1 = R:31 G:0 B:0 (identical to create solid color)
            0x00, 0x00, 0x00, 0x00, // All pixels use index 0
        ];

        // Allocate space for the decoded pixels (RGBA, 4x4 block = 16 pixels)
        let mut decoded_pixels = [0u8; 16 * 4];

        // Decode the BC1 block
        unsafe {
            rgbcx::unpack_bc1(
                bc1_block.as_ptr() as *const std::ffi::c_void,
                decoded_pixels.as_mut_ptr() as *mut std::ffi::c_void,
                true, // set_alpha
                rgbcx::bc1_approx_mode::cBC1Ideal, // mode
            );
        }

        // All pixels should be red
        for x in 0..16 {
            let pixel_idx = x * 4;
            let r = decoded_pixels[pixel_idx];
            let g = decoded_pixels[pixel_idx + 1];
            let b = decoded_pixels[pixel_idx + 2];
            let a = decoded_pixels[pixel_idx + 3];

            // Check exact values
            assert_eq!(r, 255, "Pixel {x} red value mismatch");
            assert_eq!(g, 0, "Pixel {x} green value mismatch");
            assert_eq!(b, 0, "Pixel {x} blue value mismatch");
            assert_eq!(a, 255, "Pixel {x} alpha value mismatch");
        }
    }

    #[test]
    fn can_decode_bc1_block_with_transparency() {
        // Test case with transparency (c0 < c1 for 3-color mode with alpha)
        let bc1_block: [u8; 8] = [
            0x00, 0xF0, // c0 = R:30 G:0 B:0 (intentionally less than c1)
            0x00, 0xF8, // c1 = R:31 G:0 B:0
            0xFF, 0xFF, 0xFF, 0xFF, // All pixels use index 3 (transparent)
        ];

        // Allocate space for the decoded pixels (RGBA, 4x4 block = 16 pixels)
        let mut decoded_pixels = [0u8; 16 * 4];

        // Decode the BC1 block
        unsafe {
            rgbcx::unpack_bc1(
                bc1_block.as_ptr() as *const std::ffi::c_void,
                decoded_pixels.as_mut_ptr() as *mut std::ffi::c_void,
                true, // set_alpha
                rgbcx::bc1_approx_mode::cBC1Ideal, // mode
            );
        }

        // All pixels should be transparent
        for x in 0..16 {
            let pixel_idx = x * 4;
            let r = decoded_pixels[pixel_idx];
            let g = decoded_pixels[pixel_idx + 1];
            let b = decoded_pixels[pixel_idx + 2];
            let a = decoded_pixels[pixel_idx + 3];
            
            // Check that alpha is 0 (transparent)
            assert_eq!(a, 0, "Pixel {x} expected alpha 0, got {a}");
            // For transparent pixels, RGB values should also be 0
            assert_eq!(r, 0, "Pixel {x} expected red 0, got {r}");
            assert_eq!(g, 0, "Pixel {x} expected green 0, got {g}");
            assert_eq!(b, 0, "Pixel {x} expected blue 0, got {b}");
        }
    }

    // Test that we can encode and then decode a BC1 block
    // This test uses init() since it includes encoding
    #[test]
    fn can_encode_and_decode_bc1_block() {
        // Initialize rgbcx for encoding
        unsafe { rgbcx::init(rgbcx::bc1_approx_mode::cBC1Ideal) };

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

        // Verify the decoded pixels - they won't match exactly due to lossy compression,
        // but red should still be mostly red and green should still be mostly green
        for y in 0..4 {
            for x in 0..4 {
                let idx = (y * 4 + x) * 4;
                let decoded_r = decoded_pixels[idx];
                let decoded_g = decoded_pixels[idx + 1];

                if (x + y) % 2 == 0 {
                    // Should be mostly red
                    assert!(decoded_r > decoded_g, 
                        "Expected red dominant at ({x},{y}), got R:{decoded_r}, G:{decoded_g}");
                } else {
                    // Should be mostly green
                    assert!(decoded_g > decoded_r, 
                        "Expected green dominant at ({x},{y}), got R:{decoded_r}, G:{decoded_g}");
                }
            }
        }
    }

    /// Represents a single RGBA8888 pixel color from a decoded BC1 block
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Color8888 {
        /// Red component (0-255)
        pub r: u8,
        /// Green component (0-255)
        pub g: u8,
        /// Blue component (0-255)
        pub b: u8,
        /// Alpha component (0-255)
        pub a: u8,
    }
}