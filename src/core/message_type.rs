use std::{net::Ipv4Addr, collections::HashMap};

use chrono::{Duration, NaiveTime};
use itertools::Itertools;

use super::packet_context::HardwareAddress;



pub trait PacketType: AsRef<[u8]> {

    fn from_raw_bytes(raw: &[u8]) -> Self;

}

#[derive(Clone)]
pub enum DhcpOption {

    Pad,
    End,
    SubnetMask(Vec<u8>),
    TimeOffset(Vec<u8>),
    RouterOption(Vec<u8>),
    TimeServer(Vec<u8>),
    NameServer(Vec<u8>),
    DomainNameServer(Vec<u8>),
    LogServer(Vec<u8>),
    CookieServer(Vec<u8>),
    LPRServer(Vec<u8>),
    ImpressServer(Vec<u8>),
    ResourceLocationServer(Vec<u8>),
    HostName(Vec<u8>),
    BootFileSize(Vec<u8>),
    MeritDump(Vec<u8>),
    DomainName(Vec<u8>),
    SwapServer(Vec<u8>),
    RootPath(Vec<u8>),
    ExtensionsPath(Vec<u8>),
    IPForwarding(Vec<u8>),
    NonLocalSourceRouting(Vec<u8>),
    PolicyFilter(Vec<u8>),
    MaximumDatagramReassemblySize(Vec<u8>),
    DefaultIpTTL(Vec<u8>),
    PathMTUAgingTimeout(Vec<u8>),
    PathMTUPlateauTable(Vec<u8>),
    InterfaceMTU(Vec<u8>),
    AllSubnetsAreLocal(Vec<u8>),
    BroadcastAddr(Vec<u8>),
    PerformMaskDiscovery(Vec<u8>),
    MaskSupplier(Vec<u8>),
    PerformRouterDiscovery(Vec<u8>),
    RouterSolicitationAddr(Vec<u8>),
    StaticRoute(Vec<u8>),
    TrailerEncapsulation(Vec<u8>),
    ARPCacheTimeout(Vec<u8>),
    EthernetEncapsulation(Vec<u8>),
    TcpDefaultTTL(Vec<u8>),
    TcpKeepAliveInterval(Vec<u8>),
    TcpKeepAliveGarbage(Vec<u8>),
    NetworkInformationServiceDomain(Vec<u8>),
    NetworkInformationServers(Vec<u8>),
    NetworkTimeProtocolServers(Vec<u8>),
    VendorSpecificInformation(Vec<u8>),
    NetBiosNS(Vec<u8>),
    NetBiosDatagramDistributionServer(Vec<u8>),
    NetBiosNodeType(Vec<u8>),
    NetBiosScope(Vec<u8>),
    XWindowFontServer(Vec<u8>),
    XWindowDisplayManager(Vec<u8>),
    NetworkInformationServicePlusDomain(Vec<u8>),
    NetworkInformationServicePlusServers(Vec<u8>),
    MobileIpHomeAgent(Vec<u8>),
    SMTPServer(Vec<u8>),
    POP3Server(Vec<u8>),
    NNTPServer(Vec<u8>),
    WWWServer(Vec<u8>),
    DefaultFingerServer(Vec<u8>),
    DefaultIRCServer(Vec<u8>),
    StreetTalkServer(Vec<u8>),
    STDAServer(Vec<u8>),
    RequestedIP(Vec<u8>),
    RequestedLeaseTime(Vec<u8>),
    OptionOverload(Vec<u8>),
    TFTPServerName(Vec<u8>),
    BootFileName(Vec<u8>),
    DHCPMessageType(Vec<u8>),
    ServerId(Vec<u8>),
    ParameterRequest(Vec<u8>),
    Message(Vec<u8>),
    MaximumDHCPMessageSize(Vec<u8>),
    RenewalTimeValue(Vec<u8>),
    RebindingTimeValue(Vec<u8>),
    VendorClassId(Vec<u8>),
    ClientId(Vec<u8>),

}

impl From<DhcpOption> for u8 {
    
    fn from(option: DhcpOption) -> Self {
        use DhcpOption::*;
        match option {
            Pad => 0,
            SubnetMask(_) => 1,
            TimeOffset(_) => 2,
            RouterOption(_) => 3,
            TimeServer(_) => 4,
            NameServer(_) => 5,
            DomainNameServer(_) => 6,
            LogServer(_) => 7,
            CookieServer(_) => 8,
            LPRServer(_) => 9,
            ImpressServer(_) => 10,
            ResourceLocationServer(_) => 11,
            HostName(_) => 12,
            BootFileSize(_) => 13,
            MeritDump(_) => 14,
            DomainName(_) => 15,
            SwapServer(_) => 16,
            RootPath(_) => 17,
            ExtensionsPath(_) => 18,
            IPForwarding(_) => 19,
            NonLocalSourceRouting(_) => 20,
            PolicyFilter(_) => 21,
            MaximumDatagramReassemblySize(_) => 22,
            DefaultIpTTL(_) => 23,
            PathMTUAgingTimeout(_) => 24,
            PathMTUPlateauTable(_) => 25,
            InterfaceMTU(_) => 26,
            AllSubnetsAreLocal(_) => 27,
            BroadcastAddr(_) => 28,
            PerformMaskDiscovery(_) => 29,
            MaskSupplier(_) => 30,
            PerformRouterDiscovery(_) => 31,
            RouterSolicitationAddr(_) => 32,
            StaticRoute(_) => 33,
            TrailerEncapsulation(_) => 34,
            ARPCacheTimeout(_) => 35,
            EthernetEncapsulation(_) => 36,
            TcpDefaultTTL(_) => 37,
            TcpKeepAliveInterval(_) => 38,
            TcpKeepAliveGarbage(_) => 39,
            NetworkInformationServiceDomain(_) => 40,
            NetworkInformationServers(_) => 41,
            NetworkTimeProtocolServers(_) => 42,
            VendorSpecificInformation(_) => 43,
            NetBiosNS(_) => 44,
            NetBiosDatagramDistributionServer(_) => 45,
            NetBiosNodeType(_) => 46,
            NetBiosScope(_) => 47,
            XWindowFontServer(_) => 48,
            XWindowDisplayManager(_) => 49,
            RequestedIP(_) => 50,
            RequestedLeaseTime(_) => 51,
            OptionOverload(_) => 52,
            DHCPMessageType(_) => 53,
            ServerId(_) => 54,
            ParameterRequest(_) => 55,
            Message(_) => 56,
            MaximumDHCPMessageSize(_) => 57,
            RenewalTimeValue(_) => 58,
            RebindingTimeValue(_) => 59,
            VendorClassId(_) => 60,
            ClientId(_) => 61,
            NetworkInformationServicePlusDomain(_) => 64,
            NetworkInformationServicePlusServers(_) => 65,
            TFTPServerName(_) => 66,
            BootFileName(_) => 67,
            MobileIpHomeAgent(_) => 68,
            SMTPServer(_) => 69,
            POP3Server(_) => 70,
            NNTPServer(_) => 71,
            WWWServer(_) => 72,
            DefaultFingerServer(_) => 73,
            DefaultIRCServer(_) => 74,
            StreetTalkServer(_) => 75,
            STDAServer(_) => 76,
            End => 255,

        }
    }

}

impl From<DhcpOption> for Vec<u8> {
    fn from(value: DhcpOption) -> Self {
        value.try_into().unwrap()
    }
}

impl DhcpOption {
    fn from(n: u8, bytes: Vec<u8>) -> Self {
        use DhcpOption::*;
        match n {
            0 => Pad,
            1 => SubnetMask(bytes),
            2 => TimeOffset(bytes),
            3 => RouterOption(bytes),
            4 => TimeServer(bytes),
            5 => NameServer(bytes),
            6 => DomainNameServer(bytes),
            7 => LogServer(bytes),
            8 => CookieServer(bytes),
            9 => LPRServer(bytes),
            10 => ImpressServer(bytes),
            11 => ResourceLocationServer(bytes),
            12 => HostName(bytes),
            13 => BootFileSize(bytes),
            14 => MeritDump(bytes),
            15 => DomainName(bytes),
            16 => SwapServer(bytes),
            17 => RootPath(bytes),
            18 => ExtensionsPath(bytes),
            19 => IPForwarding(bytes),
            20 => NonLocalSourceRouting(bytes),
            21 => PolicyFilter(bytes),
            22 => MaximumDatagramReassemblySize(bytes),
            23 => DefaultIpTTL(bytes),
            24 => PathMTUAgingTimeout(bytes),
            25 => PathMTUPlateauTable(bytes),
            26 => InterfaceMTU(bytes),
            27 => AllSubnetsAreLocal(bytes),
            28 => BroadcastAddr(bytes),
            29 => PerformMaskDiscovery(bytes),
            30 => MaskSupplier(bytes),
            31 => PerformRouterDiscovery(bytes),
            32 => RouterSolicitationAddr(bytes),
            33 => StaticRoute(bytes),
            34 => TrailerEncapsulation(bytes),
            35 => ARPCacheTimeout(bytes),
            36 => EthernetEncapsulation(bytes),
            37 => TcpDefaultTTL(bytes),
            38 => TcpKeepAliveInterval(bytes),
            39 => TcpKeepAliveGarbage(bytes),
            40 => NetworkInformationServiceDomain(bytes),
            41 => NetworkInformationServers(bytes),
            42 => NetworkTimeProtocolServers(bytes),
            43 => VendorSpecificInformation(bytes),
            44 => NetBiosNS(bytes),
            45 => NetBiosDatagramDistributionServer(bytes),
            46 => NetBiosNodeType(bytes),
            47 => NetBiosScope(bytes),
            48 => XWindowFontServer(bytes),
            49 => XWindowDisplayManager(bytes),
            50 => RequestedIP(bytes),
            51 => RequestedLeaseTime(bytes),
            52 => OptionOverload(bytes),
            53 => DHCPMessageType(bytes),
            54 => ServerId(bytes),
            55 => ParameterRequest(bytes),
            56 => Message(bytes),
            57 => MaximumDHCPMessageSize(bytes),
            58 => RenewalTimeValue(bytes),
            59 => RebindingTimeValue(bytes),
            60 => VendorClassId(bytes),
            61 => ClientId(bytes),
            64 => NetworkInformationServicePlusDomain(bytes),
            65 => NetworkInformationServicePlusServers(bytes),
            66 => TFTPServerName(bytes),
            67 => BootFileName(bytes),
            68 => MobileIpHomeAgent(bytes),
            69 => SMTPServer(bytes),
            70 => POP3Server(bytes),
            71 => NNTPServer(bytes),
            72 => WWWServer(bytes),
            73 => DefaultFingerServer(bytes),
            74 => DefaultIRCServer(bytes),
            75 => StreetTalkServer(bytes),
            76 => STDAServer(bytes),
            255 => End,
            _ => End
        }
    }

}

pub struct DhcpOptions {

    pub options: HashMap<u8, DhcpOption>

}

impl DhcpOptions {

    pub fn count(&self) -> u8 {
        self.options.iter().count() as u8
    } 

    pub fn add(&mut self, option: DhcpOption) {
        self.options.insert(option.clone().try_into().unwrap(), option);
    }

    pub fn is_defined(&self, option: DhcpOption) -> bool {
        self.options.contains_key(&(option.try_into().unwrap()))
    }

    pub fn empty() -> Self {
        let options : HashMap<u8, DhcpOption> = HashMap::new();
        Self { options }
    }

}

impl From<DhcpOptions> for Vec<u8> {
    fn from(value: DhcpOptions) -> Self {

        let mut buf: Vec<u8> = Vec::new();

        for option in value.options {
            let opt_vec = Vec::from(option.1);
            let opt_len: u8 = opt_vec.len() as u8;
            let mut opt_buf = Vec::new();

            opt_buf.push(option.0);
            opt_buf.push(opt_len);
            opt_buf.append(&mut Vec::from(opt_vec));

            buf.append(&mut opt_buf);

        }
        buf
    }
}

impl From<Vec<u8>> for DhcpOptions {
     fn from(mut data : Vec<u8>) -> Self{

        let mut options = DhcpOptions{ options: HashMap::new() };
        while data.len() > 0 {
            let code = data.remove(0);
            if code == 0u8 {
                continue;
            }
            if code == 255 {
                break;
            }
            let len = data.remove(0) as usize;
            let value = data.drain(0..len).as_slice().to_owned();
            options.add(DhcpOption::from(code, value));
        }
        options
    }
}

pub struct DhcpV4Packet {
    op: u8,
    htype : u8,
    hlen : u8,
    hops : u8,
    xid : u32,
    secs : NaiveTime,
    flags : [u8; 2],
    ciaddr : Ipv4Addr,
    yiaddr : Ipv4Addr,
    siaddr : Ipv4Addr,
    giaddr : Ipv4Addr,
    chadd : HardwareAddress,
    sname : String,
    file : String,
    options : DhcpOptions
}

impl DhcpV4Packet {

    pub fn get_htype(&self) -> &u8 {
        &self.htype
    } 

    pub fn set_htype(&mut self, htype: u8) {
        self.htype = htype;
    }

    pub fn empty() -> Self{
        Self {op: 0u8, htype: 0u8, hlen: 0, hops: 0, xid: 0, secs: NaiveTime::from_hms_opt(0,0,0).unwrap(), flags: [0u8; 2], ciaddr: Ipv4Addr::UNSPECIFIED, yiaddr: Ipv4Addr::UNSPECIFIED, siaddr: Ipv4Addr::UNSPECIFIED, giaddr: Ipv4Addr::UNSPECIFIED, chadd: HardwareAddress::new([0;16]), sname: String::new(), file: String::new(), options: DhcpOptions::empty() }
    }

}

impl PacketType for DhcpV4Packet {
    fn from_raw_bytes(raw : &[u8]) -> Self{
        let mut raw = raw.to_vec();
        let op = raw.remove(0);
        let htype = raw.remove(0);
        let hlen = raw.remove(0);
        let hops = raw.remove(0);
        let next:[u8; 4] = raw.drain(0..4).as_slice().to_owned().try_into().unwrap();
        let xid = u32::from_le_bytes(next);
        let next: [u8; 2] = raw.drain(0..2).as_slice().to_owned().try_into().unwrap();
        let secs = NaiveTime::from_hms_opt(0, 0, u16::from_le_bytes(next) as u32).unwrap();

        let flags = raw.drain(0..2).as_slice().to_owned().try_into().unwrap();
        let (a, b, c, d) = raw.drain(0..4).collect_tuple().unwrap();

        let ciaddr = Ipv4Addr::new(a, b, c, d);
        let (a, b, c, d) = raw.drain(0..4).collect_tuple().unwrap();

        let yiaddr = Ipv4Addr::new(a, b, c, d);
        let (a, b, c, d) = raw.drain(0..4).collect_tuple().unwrap();

        let siaddr = Ipv4Addr::new(a, b, c, d);
        let (a, b, c, d) = raw.drain(0..4).collect_tuple().unwrap();

        let giaddr = Ipv4Addr::new(a, b, c, d);
        let next: [u8; 16] = raw.drain(0..16).as_slice().to_owned().try_into().unwrap();
        let chadd = HardwareAddress::new(next);
        let next = raw.drain(0..64).as_slice().to_vec();
        let sname = String::from_utf8_lossy(&next).to_string();
        let next = raw.drain(0..128).as_slice().to_vec();
        let file = String::from_utf8_lossy(&next).to_string();
        let _magic_cookie = raw.drain(0..4).as_slice().to_vec();
        let options = DhcpOptions::from(raw); 
        Self { op, htype, hlen, hops, xid, secs, flags, ciaddr, yiaddr, siaddr, giaddr, chadd, sname, file, options }

    }

}

impl AsRef<[u8]> for DhcpV4Packet {
    fn as_ref(&self) -> &[u8] {
        todo!()
    }
}
