use crate::v12::BatchReturn;

#[test]
#[should_panic(expected = "items length 1 does not match batch size 300")]
fn misaligned_success_panics() {
    let br = BatchReturn::ok(300);
    br.successes(&["first"]);
}
