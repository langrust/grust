#![allow(warnings)]

use grust::grust;
mod macro_output;

grust! {
    #![dump = "examples/aeb/src/macro_output.rs"]

    import signal collision_collection: int;
    import signal maneuver_acknoledgement: int;
    import signal vehicle_data: int;
    import signal nvm_inp: int;
    import signal cam_obj_info: int;
    import signal fused_context_data: int;
    import signal common_variant_mngt: int;

    // TODO
}
