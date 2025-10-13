#![allow(warnings)]

use grust::grust;

grust! {
    #![dump = "grust/out/init_struct.rs"]

    enum Priority {
        Hight, Medium, Low,
    }

    struct Alarm {
        prio: Priority,
        data: int,
    }

    component delayed_alarm(alarm: Alarm) -> (delayed: Alarm) {
        init temp = Alarm { prio: Priority::Low, data: 0 };
        delayed = last temp;
        let temp: Alarm = alarm;
    }
}
