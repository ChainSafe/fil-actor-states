use fvm_shared4::error::ExitCode;

use crate::v13::{BatchReturn, BatchReturnGen};

#[test]
fn batch_generation() {
    let mut generator = BatchReturnGen::new(5);
    generator
        .add_success()
        .add_fail(ExitCode::SYS_OUT_OF_GAS)
        .add_fail(ExitCode::USR_ILLEGAL_STATE)
        .add_success()
        .add_fail(ExitCode::USR_ILLEGAL_ARGUMENT);

    let br = generator.generate();
    assert_eq!(5, br.size());
    assert!(!br.all_ok());
    assert_eq!(
        vec![
            ExitCode::OK,
            ExitCode::SYS_OUT_OF_GAS,
            ExitCode::USR_ILLEGAL_STATE,
            ExitCode::OK,
            ExitCode::USR_ILLEGAL_ARGUMENT
        ],
        br.codes()
    );

    let ret_vals = vec!["first", "second", "third", "fourth", "fifth"];
    assert_eq!(vec![&"first", &"fourth"], br.successes(&ret_vals));
}

#[test]
fn batch_generation_constants() {
    let br = BatchReturn::ok(3);
    assert_eq!(3, br.size());
    assert!(br.all_ok());
    assert_eq!(vec![ExitCode::OK, ExitCode::OK, ExitCode::OK], br.codes());
    let ret_vals = vec!["first", "second", "third"];
    assert_eq!(
        ret_vals.iter().collect::<Vec<&&str>>(),
        br.successes(&ret_vals)
    );

    let br = BatchReturn::empty();
    assert_eq!(0, br.size());
    assert!(br.all_ok());
    assert_eq!(Vec::<ExitCode>::new(), br.codes());
    let empty_successes = Vec::<u64>::new();
    assert_eq!(
        empty_successes.iter().collect::<Vec<&u64>>(),
        br.successes(&empty_successes)
    );
}

#[test]
#[should_panic(
    expected = "programmer error, mismatched batch size 3 and processed count 4 batch return must include success/fail for all inputs"
)]
fn batch_generation_programmer_error_too_many() {
    let mut generator = BatchReturnGen::new(3);
    generator
        .add_success()
        .add_success()
        .add_success()
        .add_success();

    // this will panic
    generator.generate();
}

#[test]
#[should_panic(
    expected = "programmer error, mismatched batch size 3 and processed count 2 batch return must include success/fail for all inputs"
)]
fn batch_generation_programmer_error_too_few() {
    let mut generator = BatchReturnGen::new(3);
    generator.add_success().add_fail(ExitCode::USR_NOT_FOUND);

    // this will panic
    generator.generate();
}

#[test]
#[should_panic(expected = "items length 1 does not match batch size 300")]
fn misaligned_success_panics() {
    let br = BatchReturn::ok(300);
    br.successes(&["first"]);
}
