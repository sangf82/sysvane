//! The monitor's own name and serial from its EDID, which Windows keeps in the registry. The capabilities
//! string names the scaler chip ("CVTE"), and the display number can change, so both come from here.

#[derive(Debug, PartialEq)]
pub struct Edid {
    /// The monitor name descriptor, such as "AL258Q".
    pub name: Option<String>,
    /// Manufacturer, product code and serial: the same for this monitor on any port. `None` when the maker
    /// left the serial empty or zero (the AL258Q says "0000000000000"), as two such monitors look alike.
    pub id: Option<String>,
}

impl Edid {
    /// A base EDID block (128 bytes or more). `None` when the header or size is wrong.
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 128 || bytes[..8] != [0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0] {
            return None;
        }
        let maker = u16::from_be_bytes([bytes[8], bytes[9]]);
        let maker: String = [10, 5, 0]
            .iter()
            .map(|shift| char::from(b'A' - 1 + ((maker >> shift) & 0x1F) as u8))
            .collect();
        let product = u16::from_le_bytes([bytes[10], bytes[11]]);
        let number = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let text = |tag: u8| {
            (54..126).step_by(18).find_map(|at| {
                let d = &bytes[at..at + 18];
                (d[..3] == [0, 0, 0] && d[3] == tag).then(|| {
                    let s: String = d[5..]
                        .iter()
                        .take_while(|&&b| b != 0x0A)
                        .map(|&b| char::from(b))
                        .filter(|c| c.is_ascii_graphic() || *c == ' ')
                        .collect();
                    s.trim().to_owned()
                })
            })
        };
        let real = |s: &String| s.chars().any(|c| c != '0' && c != ' ');
        let serial = text(0xFF)
            .filter(real)
            .or_else(|| Some(number.to_string()).filter(real));
        Some(Self {
            name: text(0xFC).filter(|s| !s.is_empty()),
            id: serial.map(|s| format!("{maker}{product:04X}-{s}")),
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn edid(name: &[u8], serial: &[u8]) -> Vec<u8> {
        let mut b = vec![0u8; 128];
        b[..8].copy_from_slice(&[0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0]);
        b[8..10].copy_from_slice(&[0x04, 0x72]); // "ACR"
        b[10..12].copy_from_slice(&[0x4B, 0x0A]);
        b[12..16].copy_from_slice(&7u32.to_le_bytes());
        for (at, tag, text) in [(72, 0xFC, name), (90, 0xFF, serial)] {
            b[at + 3] = tag;
            let mut field = text.to_vec();
            field.push(0x0A);
            field.resize(13, b' ');
            b[at + 5..at + 18].copy_from_slice(&field);
        }
        b
    }

    #[test]
    fn the_name_and_a_stable_id_are_read() {
        let e = Edid::parse(&edid(b"AL258Q", b"T4XAA0012345")).unwrap();
        assert_eq!(e.name.as_deref(), Some("AL258Q"));
        assert_eq!(e.id.as_deref(), Some("ACR0A4B-T4XAA0012345"));
        let e = Edid::parse(&edid(b"", b"")).unwrap();
        assert_eq!(
            (e.name, e.id.as_deref()),
            (None, Some("ACR0A4B-7")),
            "numeric serial"
        );
    }

    #[test]
    fn a_zero_serial_is_no_id() {
        let mut b = edid(b"AL258Q", b"0000000000000");
        b[12..16].fill(0);
        assert_eq!(Edid::parse(&b).unwrap().id, None);
    }

    #[test]
    fn a_broken_block_is_refused() {
        assert_eq!(Edid::parse(&[0; 128]), None);
        assert_eq!(Edid::parse(&edid(b"X", b"Y")[..100]), None);
    }
}
