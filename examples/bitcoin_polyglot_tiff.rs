use bitcoin::consensus::encode::serialize;
use bitcoin::script::PushBytes;
use bitcoin::{
    absolute, transaction, Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
};
use secp256k1::{Message, Secp256k1};
use rand::thread_rng;
use std::fs::File;
use std::io::Write;
use std::convert::TryFrom;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secp = Secp256k1::new();
    let mut rng = thread_rng();

    // 1. 8x8 Grayscale Pixel Data (64 bytes) - Fits in one OP_RETURN
    let pixel_data: [u8; 64] = [
        0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
        0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00,
        0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
        0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
        0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF,
        0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
        0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00,
        0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
    ];

    // 2. CONSTRUCT TRANSACTION
    let mut tx = Transaction {
        version: transaction::Version(1296891946), // "MM\0*"
        lock_time: absolute::LockTime::from_consensus(0),
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(546),
            script_pubkey: ScriptBuf::new_op_return(
                <&PushBytes>::try_from(&pixel_data[..]).unwrap()
            ),
        }],
    };

    // 3. SERIALIZE
    let mut tx_bytes = serialize(&tx);
    let pixel_offset = tx_bytes.windows(64).position(|w| w == pixel_data).unwrap() as u32;

    // 4. PREPARE RESOLUTION DATA (Rational: 72/1)
    // We append this after the TX so it has its own offset
    let res_offset = tx_bytes.len() as u32;
    tx_bytes.extend_from_slice(&72u32.to_be_bytes()); // Numerator
    tx_bytes.extend_from_slice(&1u32.to_be_bytes());  // Denominator

    // 5. CONSTRUCT IFD (Footer)
    let ifd_offset = tx_bytes.len() as u32;
    let mut ifd = Vec::new();
    let num_entries: u16 = 11; // Standard Baseline
    ifd.extend_from_slice(&num_entries.to_be_bytes());

    let mut add_tag = |tag: u16, typ: u16, count: u32, val: u32| {
        ifd.extend_from_slice(&tag.to_be_bytes());
        ifd.extend_from_slice(&typ.to_be_bytes());
        ifd.extend_from_slice(&count.to_be_bytes());
        ifd.extend_from_slice(&val.to_be_bytes());
    };

    // Sorted by Tag ID (Required by TIFF spec)
    add_tag(256, 3, 1, 8 << 16);          // Width
    add_tag(257, 3, 1, 8 << 16);          // Length
    add_tag(258, 3, 1, 8 << 16);          // BitsPerSample
    add_tag(259, 3, 1, 1 << 16);          // Compression
    add_tag(262, 3, 1, 1 << 16);          // Photometric
    add_tag(273, 4, 1, pixel_offset);     // StripOffsets
    add_tag(277, 3, 1, 1 << 16);          // SamplesPerPixel
    add_tag(279, 4, 1, 64);               // StripByteCounts
    add_tag(282, 5, 1, res_offset);       // XResolution (Rational)
    add_tag(283, 5, 1, res_offset);       // YResolution (Rational)
    add_tag(296, 3, 1, 2 << 16);          // ResUnit (Inch)

    ifd.extend_from_slice(&0u32.to_be_bytes());
    tx_bytes.extend(ifd);

    // Update Header Pointer
    let ptr = ifd_offset.to_be_bytes();
    tx_bytes[4..8].copy_from_slice(&ptr);

    // 6. SAVE
    let mut file = File::create("polyglot.tif")?;
    file.write_all(&tx_bytes)?;

    println!("Success! TIFF offsets: Pixel={}, IFD={}", pixel_offset, ifd_offset);
    Ok(())
}
