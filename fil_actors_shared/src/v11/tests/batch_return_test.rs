use crate::v11::BatchReturn;
use fvm_shared3::error::ExitCode;

#[test]
fn batch_generation_constants() {
    let br = BatchReturn::ok(3);
    assert_eq!(3, br.size());
    assert!(br.all_ok());
    assert_eq!(vec![ExitCode::OK, ExitCode::OK, ExitCode::OK], br.codes());
    let ret_vals = vec!["first", "second", "third"];
    assert_eq!(ret_vals, br.successes(&ret_vals));

    let br = BatchReturn::empty();
    assert_eq!(0, br.size());
    assert!(br.all_ok());
    assert_eq!(Vec::<ExitCode>::new(), br.codes());
    let empty_successes = Vec::<u64>::new();
    assert_eq!(empty_successes, br.successes(&empty_successes));
}

#[test]
#[should_panic(expected = "items length 1 does not match batch size 300")]
fn misaligned_success_panics() {
    let br = BatchReturn::ok(300);
    br.successes(&["first"]);
}
