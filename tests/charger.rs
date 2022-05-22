use umlstate::*;

#[derive(Clone)]
struct UsbConnected;
#[derive(Clone)]
struct UsbDisconnected;
#[derive(Clone)]
struct ChargeActive;
#[derive(Clone)]
struct ChargeInactive;

umlstate! {
    machine ChargePower {
        state Unpowered;

        machine Powered {
            state WaitCharge;
            state Charging;
            state ChargeDone;

            //initial -> WaitCharge;
            WaitCharge + ChargeActive => Charging / indicate(Charge::Starting);
            Charging + ChargeInactive => ChargeDone / indicate(Charge::Done);
        }

        //initial -> Unpowered / use_power(Power::Battery);
        Unpowered + UsbConnected => Powered / use_power(Power::Usb);
        Powered + UsbDisconnected => Unpowered / use_power(Power::Battery);
    }
}

#[derive(Debug)]
enum Power {
    Battery,
    Usb,
}

#[derive(Debug)]
enum Charge {
    Starting,
    Done,
}

fn use_power(source: Power) {
    println!("using power from {:?}", source);
}

fn indicate(charge: Charge) {
    println!("now {:?} charging", charge);
}

#[test]
fn charger() {
    let mut charge_logic = ChargePower::new();
    charge_logic.enter();
    charge_logic.process(UsbConnected);
    charge_logic.process(ChargeActive);
    charge_logic.process(ChargeInactive);
    charge_logic.process(ChargeActive);
    charge_logic.process(UsbDisconnected);
    charge_logic.process(ChargeInactive);
}
