use mino::{places, standard_rules::J, MatBuf};

fn main() {
    let mut mat = MatBuf::new();
    // 6 .....x.x..
    // 5 ...xxxxx..
    //   0123456789
    mat.set(5, 0b0011111000);
    mat.set(6, 0b0010100000);

    let mut result = vec![];

    for i in 0..10 {
        println!("{}/10", i + 1);
        for _ in 0..100_000 {
            result = std::hint::black_box(places(&mat, J).collect::<Vec<_>>());
        }
    }

    println!("{result:?}");
}
