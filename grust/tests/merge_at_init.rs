grust::grust! {
    #![dump = "grust/out/merge_at_init.rs"]

    import signal  measure  : float;
    import event  stabilize  : float;
    export event  compute_ev  : float;

    service kalman_task @ [10, 3000] {
        let event measure_ev : float = on_change(measure);
        compute_ev = merge(measure_ev, stabilize);
    }
}
