#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Context {}
impl Context {
    fn init() -> Context {
        Default::default()
    }
}
pub async fn run_toto_loop(
    mut collision_collection_channel: tokio::sync::mpsc::Receiver<i64>,
    mut maneuver_acknoledgement_channel: tokio::sync::mpsc::Receiver<i64>,
    mut vehicle_data_channel: tokio::sync::mpsc::Receiver<i64>,
    mut nvm_inp_channel: tokio::sync::mpsc::Receiver<i64>,
    mut cam_obj_info_channel: tokio::sync::mpsc::Receiver<i64>,
    mut fused_context_data_channel: tokio::sync::mpsc::Receiver<i64>,
    mut common_variant_mngt_channel: tokio::sync::mpsc::Receiver<i64>,
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
