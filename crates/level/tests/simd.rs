use bedrockrs_level::unpacked::UnpackedArray;

#[test]
fn simd_test() {
    // let mut words: Vec<u32> = vec![0b10101010101010101010101010101010; 4096 / BLOCKS_PER_WORD as usize];
    let mut words = vec![
        0b10101010101010101010101010101010,
        0b10000000000000000000000000000000,
        0b11000000000000000000000000000000,
        0b11100000000000000000000000000000,
        0b11110000000000000000000000000000,
        0b11111000000000000000000000000000,
        0b11111100000000000000000000000000,
        0b11111110000000000000000000000000,
        0b11111111000000000000000000000000,
    ];

    // let mut words = vec![];
    words.resize(4096, 0b11111111000000000000000000000000);

    let mut indices_simd = Box::new([0; 4096]);
    let mut indices_regular = Box::new([0; 4096]);

    unsafe {
        UnpackedArray::unpack_oct::<16>(&words, &mut indices_simd);
    }

    UnpackedArray::unpack_nonsimd(16, &words, indices_regular.as_mut_slice());

    println!("{indices_regular:?}");
    println!("{indices_simd:?}");

    assert_eq!(indices_simd, indices_regular);
}
