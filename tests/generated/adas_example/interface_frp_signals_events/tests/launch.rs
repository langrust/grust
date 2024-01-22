use json::read_json;

use interface_frp_signals_events::{launch, SystemOutput};

#[test]
fn frp_signals_events_system_should_compute_the_outputs_as_expected() {
    let input_path = "./tests/fixture/inputs.json";
    let output_path = "./tests/outputs/asynchronous_frp_outputs.json";
    let control = "./tests/fixture/outputs_expected.json";

    launch(input_path, output_path);

    // Compare outputs to the expected ones
    let outputs = read_json(output_path);
    let control = read_json(control);
    for (output, control) in outputs.zip(control) {
        let output: SystemOutput = output.unwrap();
        let control: SystemOutput = control.unwrap();
        assert_eq!(output, control)
    }
}
