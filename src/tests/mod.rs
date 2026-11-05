#[cfg(all(
    feature = "tcpudp",
    feature = "rtu",
    feature = "ascii",
    feature = "std"
))]
mod test_std;

#[cfg(not(feature = "std"))]
mod test_nostd;

mod test_features;
