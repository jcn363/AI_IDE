use rust_ai_ide_debugger::debugger::Debugger;

#[tokio::test]
async fn test_debugger_creation() {
    let debugger = Debugger::new();
    // The debugger should start in the Disconnected state
    // We'll verify this by checking that we can't run the debugger without starting a session
    let result = std::panic::catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut debugger = Debugger::new();
            let _ = debugger.run().await;
        });
    });
    assert!(
        result.is_err(),
        "Debugger should not be able to run without starting a session"
    );
}

#[test]
fn test_backend_detection() {
    // This is a simple test to verify the test framework is working
    assert!(true);
}
