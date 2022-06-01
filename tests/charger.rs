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
        state Unpowered {
            entry / use_power(Power::Battery);
        }

        state Powered {
            state WaitCharge;
            state Charging;
            state ChargeDone;

            entry / use_power(Power::Usb);

            <*> => WaitCharge;
            WaitCharge + ChargeActive => Charging / indicate(Charge::Starting);
            Charging + ChargeInactive => ChargeDone / indicate(Charge::Done);
        }

        <*> => Unpowered;
        Unpowered + UsbConnected => Powered;
        Powered + UsbDisconnected => Unpowered;

        ChargeActive / println!("ignoring charge active signal");
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
    charge_logic.process(ChargeActive);
    charge_logic.process(ChargeInactive);
    charge_logic.process(UsbConnected);
    charge_logic.process(ChargeActive);
    charge_logic.process(ChargeInactive);
    charge_logic.process(ChargeActive);
    charge_logic.process(UsbDisconnected);
    charge_logic.process(ChargeInactive);
}
