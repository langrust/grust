pub async fn run_toto_loop(
    collision_collection_channel: tokio::sync::mpsc::Receiver<i64>,
    maneuver_acknoledgement_channel: tokio::sync::mpsc::Receiver<i64>,
    vehicle_data_channel: tokio::sync::mpsc::Receiver<i64>,
    nvm_inp_channel: tokio::sync::mpsc::Receiver<i64>,
    cam_obj_info_channel: tokio::sync::mpsc::Receiver<i64>,
    fused_context_data_channel: tokio::sync::mpsc::Receiver<i64>,
    common_variant_mngt_channel: tokio::sync::mpsc::Receiver<i64>,
) -> () {
    let context = Context::init();
    loop {
        tokio::select! {
            collision_collection = collision_collection_channel.recv() => {}
            maneuver_acknoledgement = maneuver_acknoledgement_channel.recv()
            => {} vehicle_data = vehicle_data_channel.recv() => {} nvm_inp =
            nvm_inp_channel.recv() => {} cam_obj_info =
            cam_obj_info_channel.recv() => {} fused_context_data =
            fused_context_data_channel.recv() => {} common_variant_mngt =
            common_variant_mngt_channel.recv() => {}
        }
    }
}
