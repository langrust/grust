#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Context {}
impl Context {
    fn init() -> Context {
        Default::default()
    }
}
pub struct TotoService {
    context: Context,
    collision_collection_channel: tokio::sync::mpsc::Receiver<i64>,
    maneuver_acknoledgement_channel: tokio::sync::mpsc::Receiver<i64>,
    vehicle_data_channel: tokio::sync::mpsc::Receiver<i64>,
    nvm_inp_channel: tokio::sync::mpsc::Receiver<i64>,
    cam_obj_info_channel: tokio::sync::mpsc::Receiver<i64>,
    fused_context_data_channel: tokio::sync::mpsc::Receiver<i64>,
    common_variant_mngt_channel: tokio::sync::mpsc::Receiver<i64>,
}
impl TotoService {
    fn new(
        collision_collection_channel: tokio::sync::mpsc::Receiver<i64>,
        maneuver_acknoledgement_channel: tokio::sync::mpsc::Receiver<i64>,
        vehicle_data_channel: tokio::sync::mpsc::Receiver<i64>,
        nvm_inp_channel: tokio::sync::mpsc::Receiver<i64>,
        cam_obj_info_channel: tokio::sync::mpsc::Receiver<i64>,
        fused_context_data_channel: tokio::sync::mpsc::Receiver<i64>,
        common_variant_mngt_channel: tokio::sync::mpsc::Receiver<i64>,
    ) -> TotoService {
        let context = Context::init();
        TotoService {
            context,
            collision_collection_channel,
            maneuver_acknoledgement_channel,
            vehicle_data_channel,
            nvm_inp_channel,
            cam_obj_info_channel,
            fused_context_data_channel,
            common_variant_mngt_channel,
        }
    }
    pub async fn run_loop(
        collision_collection_channel: tokio::sync::mpsc::Receiver<i64>,
        maneuver_acknoledgement_channel: tokio::sync::mpsc::Receiver<i64>,
        vehicle_data_channel: tokio::sync::mpsc::Receiver<i64>,
        nvm_inp_channel: tokio::sync::mpsc::Receiver<i64>,
        cam_obj_info_channel: tokio::sync::mpsc::Receiver<i64>,
        fused_context_data_channel: tokio::sync::mpsc::Receiver<i64>,
        common_variant_mngt_channel: tokio::sync::mpsc::Receiver<i64>,
    ) {
        let mut service = TotoService::new(
            collision_collection_channel,
            maneuver_acknoledgement_channel,
            vehicle_data_channel,
            nvm_inp_channel,
            cam_obj_info_channel,
            fused_context_data_channel,
            common_variant_mngt_channel,
        );
        loop {
            tokio::select! {
                collision_collection =
                service.collision_collection_channel.recv() =>
                service.handle_collision_collection(collision_collection.unwrap()).await,
                maneuver_acknoledgement =
                service.maneuver_acknoledgement_channel.recv() =>
                service.handle_maneuver_acknoledgement(maneuver_acknoledgement.unwrap()).await,
                vehicle_data = service.vehicle_data_channel.recv() =>
                service.handle_vehicle_data(vehicle_data.unwrap()).await,
                nvm_inp = service.nvm_inp_channel.recv() =>
                service.handle_nvm_inp(nvm_inp.unwrap()).await, cam_obj_info =
                service.cam_obj_info_channel.recv() =>
                service.handle_cam_obj_info(cam_obj_info.unwrap()).await,
                fused_context_data = service.fused_context_data_channel.recv()
                =>
                service.handle_fused_context_data(fused_context_data.unwrap()).await,
                common_variant_mngt =
                service.common_variant_mngt_channel.recv() =>
                service.handle_common_variant_mngt(common_variant_mngt.unwrap()).await,
            }
        }
    }
    async fn handle_collision_collection(&mut self, collision_collection: i64) {}
    async fn handle_maneuver_acknoledgement(&mut self, maneuver_acknoledgement: i64) {}
    async fn handle_vehicle_data(&mut self, vehicle_data: i64) {}
    async fn handle_nvm_inp(&mut self, nvm_inp: i64) {}
    async fn handle_cam_obj_info(&mut self, cam_obj_info: i64) {}
    async fn handle_fused_context_data(&mut self, fused_context_data: i64) {}
    async fn handle_common_variant_mngt(&mut self, common_variant_mngt: i64) {}
}
