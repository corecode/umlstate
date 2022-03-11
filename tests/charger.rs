// use umlstate::*;

// umlstate! {
//     machine ChargePower {
//         event UsbConnected;
//         event UsbDisconnected;
//         event ChargeActive;
//         event ChargeInactive;

//         state Unpowered;
//         machine Powered {
//             state WaitCharge;
//             state Charging;
//             state ChargeDone;

//             initial -> WaitCharge;
//             WaitCharge + ChargeActive -> Charging;
//             Charging + ChargeInactive -> ChargeDone;
//         }

//         initial -> Unpowered / use_power(Power::Battery);
//         Unpowered + UsbConnected -> Powered / use_power(Power::Usb);
//         Powered + UsbDisconnected -> Unpowered / use_power(Power::Battery);
//     }
// }

#[test]
fn compiles() {}
