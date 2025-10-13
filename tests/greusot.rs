use codespan_reporting::files::SimpleFiles;

use grustine::generate_rust_project;

#[test]
fn generate_rust_project_for_greusot_test() {
    let mut files = SimpleFiles::new();

    let greusot_test_id = files.add(
        "greusot_test.gr",
        std::fs::read_to_string("tests/fixture/greusot/success/greusot_test.gr")
            .expect("unkown file"),
    );

    generate_rust_project(greusot_test_id, &mut files, "tests/generated/greusot/")
}

#[test]
fn generate_rust_project_for_contracts_test() {
    let mut files = SimpleFiles::new();

    let contracts_test_id = files.add(
        "contracts_test.gr",
        std::fs::read_to_string("tests/fixture/greusot/success/contracts_test.gr")
            .expect("unkown file"),
    );

    generate_rust_project(contracts_test_id, &mut files, "tests/generated/greusot/")
}

#[test]
fn generate_rust_project_for_contract_dependencies_transitive() {
    let mut files = SimpleFiles::new();

    let contract_dependencies_transitive_id = files.add(
        "contract_dependencies_transitive.gr",
        std::fs::read_to_string(
            "tests/fixture/greusot/success/contract_dependencies_transitive.gr",
        )
        .expect("unkown file"),
    );

    generate_rust_project(
        contract_dependencies_transitive_id,
        &mut files,
        "tests/generated/greusot/",
    )
}

#[test]
fn generate_rust_project_for_contract_dependencies_propagation() {
    let mut files = SimpleFiles::new();

    let contract_dependencies_propagation_id = files.add(
        "contract_dependencies_propagation.gr",
        std::fs::read_to_string(
            "tests/fixture/greusot/success/contract_dependencies_propagation.gr",
        )
        .expect("unkown file"),
    );

    generate_rust_project(
        contract_dependencies_propagation_id,
        &mut files,
        "tests/generated/greusot/",
    )
}

#[test]
fn generate_rust_project_for_contract_dependencies_propagation_proof() {
    let mut files = SimpleFiles::new();

    let contract_dependencies_propagation_proof_id = files.add(
        "contract_dependencies_propagation_proof.gr",
        std::fs::read_to_string(
            "tests/fixture/greusot/success/contract_dependencies_propagation_proof.gr",
        )
        .expect("unkown file"),
    );

    generate_rust_project(
        contract_dependencies_propagation_proof_id,
        &mut files,
        "tests/generated/greusot/",
    )
}
