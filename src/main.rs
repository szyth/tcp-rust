use tun_tap::Iface;

fn main() {
    let nic = Iface::new("tun0", tun_tap::Mode::Tun).expect("failed to connect");
}
