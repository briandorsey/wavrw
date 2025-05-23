//! `fmt ` Format of the audio samples in `data` chunk. (WAVEFORMATEX)  [RIFF1991](https://wavref.til.cafe/chunk/fmt/), [RIFF1994](https://wavref.til.cafe/chunk/fmt/)
//!
//! This is a fiddly chunk to parse, since the fields vary depending on the [`FormatTag`].
//! Currently only the most common formats are implemented with specific structs to parse them. Others will be parsed by [`FmtExtended`].

use core::fmt::{Display, Formatter};

use binrw::binrw;
use itertools::Itertools;
use num_enum::{FromPrimitive, IntoPrimitive};

use crate::{FourCC, KnownChunk, KnownChunkID, Summarizable};

/// A number indicating the WAVE format category of the file.
///
/// The content of the format-specific-fields [ed: everything after block_align]
/// portion of the fmt chunk, and the interpretation of the waveform data, depend on
/// this value. [RIFF1991](https://wavref.til.cafe/chunk/fmt/)
#[allow(dead_code, missing_docs)]
#[binrw]
#[brw(little, repr = u16)]
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive)]
pub enum FormatTag {
    Unknown = 0x0000,
    Pcm = 0x0001,
    Adpcm = 0x0002,
    IeeeFloat = 0x0003,
    Vselp = 0x0004,
    IbmCvsd = 0x0005,
    Alaw = 0x0006,
    Mulaw = 0x0007,
    Dts = 0x0008,
    Drm = 0x0009,
    Wmavoice9 = 0x000A,
    Wmavoice10 = 0x000B,
    OkiAdpcm = 0x0010,
    DviAdpcm = 0x0011,
    MediaspaceAdpcm = 0x0012,
    SierraAdpcm = 0x0013,
    G723Adpcm = 0x0014,
    Digistd = 0x0015,
    Digifix = 0x0016,
    DialogicOkiAdpcm = 0x0017,
    MediavisionAdpcm = 0x0018,
    CuCodec = 0x0019,
    HpDynVoice = 0x001A,
    YamahaAdpcm = 0x0020,
    Sonarc = 0x0021,
    DspgroupTruespeech = 0x0022,
    Echosc1 = 0x0023,
    AudiofileAf36 = 0x0024,
    Aptx = 0x0025,
    AudiofileAf10 = 0x0026,
    Prosody1612 = 0x0027,
    Lrc = 0x0028,
    DolbyAc2 = 0x0030,
    Gsm610x31 = 0x0031,
    Msnaudio = 0x0032,
    AntexAdpcme = 0x0033,
    ControlResVqlpc = 0x0034,
    Digireal = 0x0035,
    Digiadpcm = 0x0036,
    ControlResCr10 = 0x0037,
    NmsVbxadpcm = 0x0038,
    CsImaadpcm = 0x0039,
    Echosc3 = 0x003A,
    RockwellAdpcm = 0x003B,
    RockwellDigitalk = 0x003C,
    Xebec = 0x003D,
    G721Adpcm = 0x0040,
    G728Celp = 0x0041,
    Msg723 = 0x0042,
    IntelG7231 = 0x0043,
    IntelG729 = 0x0044,
    SharpG726 = 0x0045,
    Mpeg = 0x0050,
    Rt24 = 0x0052,
    Pac = 0x0053,
    Mpeglayer3 = 0x0055,
    LucentG723 = 0x0059,
    Cirrus = 0x0060,
    Espcm = 0x0061,
    Voxware = 0x0062,
    CanopusAtrac = 0x0063,
    ApicomG726Adpcm = 0x0064,
    ApicomG722Adpcm = 0x0065,
    Dsat = 0x0066,
    DsatDisplay = 0x0067,
    VoxwareByteAligned = 0x0069,
    VoxwareAc8 = 0x0070,
    VoxwareAc10 = 0x0071,
    VoxwareAc16 = 0x0072,
    VoxwareAc20 = 0x0073,
    VoxwareRt24 = 0x0074,
    VoxwareRt29 = 0x0075,
    VoxwareRt29Hw = 0x0076,
    VoxwareVr12 = 0x0077,
    VoxwareVr18 = 0x0078,
    VoxwareTq40 = 0x0079,
    VoxwareSc3 = 0x007A,
    VoxwareSc31 = 0x007B,
    Softsound = 0x0080,
    VoxwareTq60 = 0x0081,
    Msrt24 = 0x0082,
    G729A = 0x0083,
    MviMvi2 = 0x0084,
    DfG726 = 0x0085,
    DfGsm610 = 0x0086,
    Isiaudio = 0x0088,
    Onlive = 0x0089,
    MultitudeFtSx20 = 0x008A,
    InfocomItsG721Adpcm = 0x008B,
    ConvediaG729 = 0x008C,
    Congruency = 0x008D,
    Sbc24 = 0x0091,
    DolbyAc3Spdif = 0x0092,
    MediasonicG723 = 0x0093,
    Prosody8Kbps = 0x0094,
    ZyxelAdpcm = 0x0097,
    PhilipsLpcbb = 0x0098,
    Packed = 0x0099,
    MaldenPhonytalk = 0x00A0,
    RacalRecorderGsm = 0x00A1,
    RacalRecorderG720A = 0x00A2,
    RacalRecorderG7231 = 0x00A3,
    RacalRecorderTetraAcelp = 0x00A4,
    NecAac = 0x00B0,
    RawAac1 = 0x00FF,
    RhetorexAdpcm = 0x0100,
    Irat = 0x0101,
    VivoG723 = 0x0111,
    VivoSiren = 0x0112,
    PhilipsCelp = 0x0120,
    PhilipsGrundig = 0x0121,
    DigitalG723 = 0x0123,
    SanyoLdAdpcm = 0x0125,
    SiprolabAceplnet = 0x0130,
    SiprolabAcelp4800 = 0x0131,
    SiprolabAcelp8V3 = 0x0132,
    SiprolabG729 = 0x0133,
    SiprolabG729A = 0x0134,
    SiprolabKelvin = 0x0135,
    VoiceageAmr = 0x0136,
    DictaphoneG726Adpcm = 0x0140,
    DictaphoneCelp68 = 0x0141,
    DictaphoneCelp54 = 0x0142,
    QualcommPurevoice = 0x0150,
    QualcommHalfrate = 0x0151,
    Tubgsm = 0x0155,
    Msaudio1 = 0x0160,
    Wmaudio2 = 0x0161,
    Wmaudio3 = 0x0162,
    WmaudioLossless = 0x0163,
    Wmaspdif = 0x0164,
    UnisysNapAdpcm = 0x0170,
    UnisysNapUlaw = 0x0171,
    UnisysNapAlaw = 0x0172,
    UnisysNap16K = 0x0173,
    SycomAcmSyc008 = 0x0174,
    SycomAcmSyc701G726L = 0x0175,
    SycomAcmSyc701Celp54 = 0x0176,
    SycomAcmSyc701Celp68 = 0x0177,
    KnowledgeAdventureAdpcm = 0x0178,
    FraunhoferIisMpeg2Aac = 0x0180,
    DtsDs = 0x0190,
    CreativeAdpcm = 0x0200,
    CreativeFastspeech8 = 0x0202,
    CreativeFastspeech10 = 0x0203,
    UherAdpcm = 0x0210,
    UleadDvAudio = 0x0215,
    UleadDvAudio1 = 0x0216,
    Quarterdeck = 0x0220,
    IlinkVc = 0x0230,
    RawSport = 0x0240,
    EsstAc3 = 0x0241,
    GenericPassthru = 0x0249,
    IpiHsx = 0x0250,
    IpiRpelp = 0x0251,
    Cs2 = 0x0260,
    SonyScx = 0x0270,
    SonyScy = 0x0271,
    SonyAtrac3 = 0x0272,
    SonySpc = 0x0273,
    TelumAudio = 0x0280,
    TelumIaAudio = 0x0281,
    NorcomVoiceSystemsAdpcm = 0x0285,
    FmTownsSnd = 0x0300,
    Micronas = 0x0350,
    MicronasCelp833 = 0x0351,
    BtvDigital = 0x0400,
    IntelMusicCoder = 0x0401,
    IndeoAudio = 0x0402,
    QdesignMusic = 0x0450,
    On2Vp7Audio = 0x0500,
    On2Vp6Audio = 0x0501,
    VmeVmpcm = 0x0680,
    Tpc = 0x0681,
    LightwaveLossless = 0x08AE,
    Oligsm = 0x1000,
    Oliadpcm = 0x1001,
    Olicelp = 0x1002,
    Olisbc = 0x1003,
    Oliopr = 0x1004,
    LhCodec = 0x1100,
    LhCodecCelp = 0x1101,
    LhCodecSbc8 = 0x1102,
    LhCodecSbc12 = 0x1103,
    LhCodecSbc16 = 0x1104,
    Norris = 0x1400,
    Isiaudio2 = 0x1401,
    SoundspaceMusicompress = 0x1500,
    MpegAdtsAac = 0x1600,
    MpegRawAac = 0x1601,
    MpegLoas = 0x1602,
    NokiaMpegAdtsAac = 0x1608,
    NokiaMpegRawAac = 0x1609,
    VodafoneMpegAdtsAac = 0x160A,
    VodafoneMpegRawAac = 0x160B,
    MpegHeaac = 0x1610,
    VoxwareRt24Speech = 0x181C,
    SonicfoundryLossless = 0x1971,
    InningsTelecomAdpcm = 0x1979,
    LucentSx8300P = 0x1C07,
    LucentSx5363S = 0x1C0C,
    Cuseeme = 0x1F03,
    NtcsoftAlf2CmAcm = 0x1FC4,
    Dvm = 0x2000,
    Dts2 = 0x2001,
    Makeavis = 0x3313,
    DivioMpeg4Aac = 0x4143,
    NokiaAdaptiveMultirate = 0x4201,
    DivioG726 = 0x4243,
    LeadSpeech = 0x434C,
    LeadVorbis = 0x564C,
    WavpackAudio = 0x5756,
    Alac = 0x6C61,
    OggVorbisMode1 = 0x674F,
    OggVorbisMode2 = 0x6750,
    OggVorbisMode3 = 0x6751,
    OggVorbisMode1Plus = 0x676F,
    OggVorbisMode2Plus = 0x6770,
    OggVorbisMode3Plus = 0x6771,
    ThreeComNbx = 0x7000,
    Opus = 0x704F,
    FaadAac = 0x706D,
    AmrNb = 0x7361,
    AmrWb = 0x7362,
    AmrWp = 0x7363,
    GsmAmrCbr = 0x7A21,
    GsmAmrVbrSid = 0x7A22,
    ComverseInfosysG7231 = 0xA100,
    ComverseInfosysAvqsbc = 0xA101,
    ComverseInfosysSbc = 0xA102,
    SymbolG729A = 0xA103,
    VoiceageAmrWb = 0xA104,
    IngenientG726 = 0xA105,
    Mpeg4Aac = 0xA106,
    EncoreG726 = 0xA107,
    ZollAsao = 0xA108,
    SpeexVoice = 0xA109,
    VianixMasc = 0xA10A,
    Wm9SpectrumAnalyzer = 0xA10B,
    WmfSpectrumAnayzer = 0xA10C,
    Gsm610 = 0xA10D,
    Gsm620 = 0xA10E,
    Gsm660 = 0xA10F,
    Gsm690 = 0xA110,
    GsmAdaptiveMultirateWb = 0xA111,
    PolycomG722 = 0xA112,
    PolycomG728 = 0xA113,
    PolycomG729A = 0xA114,
    PolycomSiren = 0xA115,
    GlobalIpIlbc = 0xA116,
    RadiotimeTimeShiftRadio = 0xA117,
    NiceAca = 0xA118,
    NiceAdpcm = 0xA119,
    VocordG721 = 0xA11A,
    VocordG726 = 0xA11B,
    VocordG7221 = 0xA11C,
    VocordG728 = 0xA11D,
    VocordG729 = 0xA11E,
    VocordG729A = 0xA11F,
    VocordG7231 = 0xA120,
    VocordLbc = 0xA121,
    NiceG728 = 0xA122,
    FraceTelecomG729 = 0xA123,
    Codian = 0xA124,
    DolbyAc4 = 0xAC40,
    Flac = 0xF1AC,
    Extensible = 0xFFFE,
    Development = 0xFFFF,
    #[num_enum(catch_all)]
    Other(u16) = 0xABCD,
}

#[allow(clippy::enum_glob_use)]
impl Display for FormatTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        use FormatTag::*;
        let output = match self {
            Unknown => "WAVE_FORMAT_UNKNOWN",
            Pcm => "WAVE_FORMAT_PCM",
            Adpcm => "WAVE_FORMAT_ADPCM",
            IeeeFloat => "WAVE_FORMAT_IEEE_FLOAT",
            Vselp => "WAVE_FORMAT_VSELP",
            IbmCvsd => "WAVE_FORMAT_IBM_CVSD",
            Alaw => "WAVE_FORMAT_ALAW",
            Mulaw => "WAVE_FORMAT_MULAW",
            Dts => "WAVE_FORMAT_DTS",
            Drm => "WAVE_FORMAT_DRM",
            Wmavoice9 => "WAVE_FORMAT_WMAVOICE9",
            Wmavoice10 => "WAVE_FORMAT_WMAVOICE10",
            OkiAdpcm => "WAVE_FORMAT_OKI_ADPCM",
            DviAdpcm => "WAVE_FORMAT_DVI_ADPCM",
            MediaspaceAdpcm => "WAVE_FORMAT_MEDIASPACE_ADPCM",
            SierraAdpcm => "WAVE_FORMAT_SIERRA_ADPCM",
            G723Adpcm => "WAVE_FORMAT_G723_ADPCM",
            Digistd => "WAVE_FORMAT_DIGISTD",
            Digifix => "WAVE_FORMAT_DIGIFIX",
            DialogicOkiAdpcm => "WAVE_FORMAT_DIALOGIC_OKI_ADPCM",
            MediavisionAdpcm => "WAVE_FORMAT_MEDIAVISION_ADPCM",
            CuCodec => "WAVE_FORMAT_CU_CODEC",
            HpDynVoice => "WAVE_FORMAT_HP_DYN_VOICE",
            YamahaAdpcm => "WAVE_FORMAT_YAMAHA_ADPCM",
            Sonarc => "WAVE_FORMAT_SONARC",
            DspgroupTruespeech => "WAVE_FORMAT_DSPGROUP_TRUESPEECH",
            Echosc1 => "WAVE_FORMAT_ECHOSC1",
            AudiofileAf36 => "WAVE_FORMAT_AUDIOFILE_AF36",
            Aptx => "WAVE_FORMAT_APTX",
            AudiofileAf10 => "WAVE_FORMAT_AUDIOFILE_AF10",
            Prosody1612 => "WAVE_FORMAT_PROSODY_1612",
            Lrc => "WAVE_FORMAT_LRC",
            DolbyAc2 => "WAVE_FORMAT_DOLBY_AC2",
            Gsm610x31 => "WAVE_FORMAT_GSM610",
            Msnaudio => "WAVE_FORMAT_MSNAUDIO",
            AntexAdpcme => "WAVE_FORMAT_ANTEX_ADPCME",
            ControlResVqlpc => "WAVE_FORMAT_CONTROL_RES_VQLPC",
            Digireal => "WAVE_FORMAT_DIGIREAL",
            Digiadpcm => "WAVE_FORMAT_DIGIADPCM",
            ControlResCr10 => "WAVE_FORMAT_CONTROL_RES_CR10",
            NmsVbxadpcm => "WAVE_FORMAT_NMS_VBXADPCM",
            CsImaadpcm => "WAVE_FORMAT_CS_IMAADPCM",
            Echosc3 => "WAVE_FORMAT_ECHOSC3",
            RockwellAdpcm => "WAVE_FORMAT_ROCKWELL_ADPCM",
            RockwellDigitalk => "WAVE_FORMAT_ROCKWELL_DIGITALK",
            Xebec => "WAVE_FORMAT_XEBEC",
            G721Adpcm => "WAVE_FORMAT_G721_ADPCM",
            G728Celp => "WAVE_FORMAT_G728_CELP",
            Msg723 => "WAVE_FORMAT_MSG723",
            IntelG7231 => "WAVE_FORMAT_INTEL_G723_1",
            IntelG729 => "WAVE_FORMAT_INTEL_G729",
            SharpG726 => "WAVE_FORMAT_SHARP_G726",
            Mpeg => "WAVE_FORMAT_MPEG",
            Rt24 => "WAVE_FORMAT_RT24",
            Pac => "WAVE_FORMAT_PAC",
            Mpeglayer3 => "WAVE_FORMAT_MPEGLAYER3",
            LucentG723 => "WAVE_FORMAT_LUCENT_G723",
            Cirrus => "WAVE_FORMAT_CIRRUS",
            Espcm => "WAVE_FORMAT_ESPCM",
            Voxware => "WAVE_FORMAT_VOXWARE",
            CanopusAtrac => "WAVE_FORMAT_CANOPUS_ATRAC",
            ApicomG726Adpcm => "WAVE_FORMAT_G726_ADPCM",
            ApicomG722Adpcm => "WAVE_FORMAT_G722_ADPCM",
            Dsat => "WAVE_FORMAT_DSAT",
            DsatDisplay => "WAVE_FORMAT_DSAT_DISPLAY",
            VoxwareByteAligned => "WAVE_FORMAT_VOXWARE_BYTE_ALIGNED",
            VoxwareAc8 => "WAVE_FORMAT_VOXWARE_AC8",
            VoxwareAc10 => "WAVE_FORMAT_VOXWARE_AC10",
            VoxwareAc16 => "WAVE_FORMAT_VOXWARE_AC16",
            VoxwareAc20 => "WAVE_FORMAT_VOXWARE_AC20",
            VoxwareRt24 => "WAVE_FORMAT_VOXWARE_RT24",
            VoxwareRt29 => "WAVE_FORMAT_VOXWARE_RT29",
            VoxwareRt29Hw => "WAVE_FORMAT_VOXWARE_RT29HW",
            VoxwareVr12 => "WAVE_FORMAT_VOXWARE_VR12",
            VoxwareVr18 => "WAVE_FORMAT_VOXWARE_VR18",
            VoxwareTq40 => "WAVE_FORMAT_VOXWARE_TQ40",
            VoxwareSc3 => "WAVE_FORMAT_VOXWARE_SC3",
            VoxwareSc31 => "WAVE_FORMAT_VOXWARE_SC3_1",
            Softsound => "WAVE_FORMAT_SOFTSOUND",
            VoxwareTq60 => "WAVE_FORMAT_VOXWARE_TQ60",
            Msrt24 => "WAVE_FORMAT_MSRT24",
            G729A => "WAVE_FORMAT_G729A",
            MviMvi2 => "WAVE_FORMAT_MVI_MVI2",
            DfG726 => "WAVE_FORMAT_DF_G726",
            DfGsm610 => "WAVE_FORMAT_DF_GSM610",
            Isiaudio => "WAVE_FORMAT_ISIAUDIO",
            Onlive => "WAVE_FORMAT_ONLIVE",
            MultitudeFtSx20 => "WAVE_FORMAT_MULTITUDE_FT_SX20",
            InfocomItsG721Adpcm => "WAVE_FORMAT_INFOCOM_ITS_G721_ADPCM",
            ConvediaG729 => "WAVE_FORMAT_CONVEDIA_G729",
            Congruency => "WAVE_FORMAT_CONGRUENCY",
            Sbc24 => "WAVE_FORMAT_SBC24",
            DolbyAc3Spdif => "WAVE_FORMAT_DOLBY_AC3_SPDIF",
            MediasonicG723 => "WAVE_FORMAT_MEDIASONIC_G723",
            Prosody8Kbps => "WAVE_FORMAT_PROSODY_8KBPS",
            ZyxelAdpcm => "WAVE_FORMAT_ZYXEL_ADPCM",
            PhilipsLpcbb => "WAVE_FORMAT_PHILIPS_LPCBB",
            Packed => "WAVE_FORMAT_PACKED",
            MaldenPhonytalk => "WAVE_FORMAT_MALDEN_PHONYTALK",
            RacalRecorderGsm => "WAVE_FORMAT_RACAL_RECORDER_GSM",
            RacalRecorderG720A => "WAVE_FORMAT_RACAL_RECORDER_G720_A",
            RacalRecorderG7231 => "WAVE_FORMAT_RACAL_RECORDER_G723_1",
            RacalRecorderTetraAcelp => "WAVE_FORMAT_RACAL_RECORDER_TETRA_ACELP",
            NecAac => "WAVE_FORMAT_NEC_AAC",
            RawAac1 => "WAVE_FORMAT_RAW_AAC1",
            RhetorexAdpcm => "WAVE_FORMAT_RHETOREX_ADPCM",
            Irat => "WAVE_FORMAT_IRAT",
            VivoG723 => "WAVE_FORMAT_VIVO_G723",
            VivoSiren => "WAVE_FORMAT_VIVO_SIREN",
            PhilipsCelp => "WAVE_FORMAT_PHILIPS_CELP",
            PhilipsGrundig => "WAVE_FORMAT_PHILIPS_GRUNDIG",
            DigitalG723 => "WAVE_FORMAT_DIGITAL_G723",
            SanyoLdAdpcm => "WAVE_FORMAT_SANYO_LD_ADPCM",
            SiprolabAceplnet => "WAVE_FORMAT_SIPROLAB_ACEPLNET",
            SiprolabAcelp4800 => "WAVE_FORMAT_SIPROLAB_ACELP4800",
            SiprolabAcelp8V3 => "WAVE_FORMAT_SIPROLAB_ACELP8V3",
            SiprolabG729 => "WAVE_FORMAT_SIPROLAB_G729",
            SiprolabG729A => "WAVE_FORMAT_SIPROLAB_G729A",
            SiprolabKelvin => "WAVE_FORMAT_SIPROLAB_KELVIN",
            VoiceageAmr => "WAVE_FORMAT_VOICEAGE_AMR",
            DictaphoneG726Adpcm => "WAVE_FORMAT_G726ADPCM",
            DictaphoneCelp68 => "WAVE_FORMAT_DICTAPHONE_CELP68",
            DictaphoneCelp54 => "WAVE_FORMAT_DICTAPHONE_CELP54",
            QualcommPurevoice => "WAVE_FORMAT_QUALCOMM_PUREVOICE",
            QualcommHalfrate => "WAVE_FORMAT_QUALCOMM_HALFRATE",
            Tubgsm => "WAVE_FORMAT_TUBGSM",
            Msaudio1 => "WAVE_FORMAT_MSAUDIO1",
            Wmaudio2 => "WAVE_FORMAT_WMAUDIO2",
            Wmaudio3 => "WAVE_FORMAT_WMAUDIO3",
            WmaudioLossless => "WAVE_FORMAT_WMAUDIO_LOSSLESS",
            Wmaspdif => "WAVE_FORMAT_WMASPDIF",
            UnisysNapAdpcm => "WAVE_FORMAT_UNISYS_NAP_ADPCM",
            UnisysNapUlaw => "WAVE_FORMAT_UNISYS_NAP_ULAW",
            UnisysNapAlaw => "WAVE_FORMAT_UNISYS_NAP_ALAW",
            UnisysNap16K => "WAVE_FORMAT_UNISYS_NAP_16K",
            SycomAcmSyc008 => "WAVE_FORMAT_SYCOM_ACM_SYC008",
            SycomAcmSyc701G726L => "WAVE_FORMAT_SYCOM_ACM_SYC701_G726L",
            SycomAcmSyc701Celp54 => "WAVE_FORMAT_SYCOM_ACM_SYC701_CELP54",
            SycomAcmSyc701Celp68 => "WAVE_FORMAT_SYCOM_ACM_SYC701_CELP68",
            KnowledgeAdventureAdpcm => "WAVE_FORMAT_KNOWLEDGE_ADVENTURE_ADPCM",
            FraunhoferIisMpeg2Aac => "WAVE_FORMAT_FRAUNHOFER_IIS_MPEG2_AAC",
            DtsDs => "WAVE_FORMAT_DTS_DS",
            CreativeAdpcm => "WAVE_FORMAT_CREATIVE_ADPCM",
            CreativeFastspeech8 => "WAVE_FORMAT_CREATIVE_FASTSPEECH8",
            CreativeFastspeech10 => "WAVE_FORMAT_CREATIVE_FASTSPEECH10",
            UherAdpcm => "WAVE_FORMAT_UHER_ADPCM",
            UleadDvAudio => "WAVE_FORMAT_ULEAD_DV_AUDIO",
            UleadDvAudio1 => "WAVE_FORMAT_ULEAD_DV_AUDIO_1",
            Quarterdeck => "WAVE_FORMAT_QUARTERDECK",
            IlinkVc => "WAVE_FORMAT_ILINK_VC",
            RawSport => "WAVE_FORMAT_RAW_SPORT",
            EsstAc3 => "WAVE_FORMAT_ESST_AC3",
            GenericPassthru => "WAVE_FORMAT_GENERIC_PASSTHRU",
            IpiHsx => "WAVE_FORMAT_IPI_HSX",
            IpiRpelp => "WAVE_FORMAT_IPI_RPELP",
            Cs2 => "WAVE_FORMAT_CS2",
            SonyScx => "WAVE_FORMAT_SONY_SCX",
            SonyScy => "WAVE_FORMAT_SONY_SCY",
            SonyAtrac3 => "WAVE_FORMAT_SONY_ATRAC3",
            SonySpc => "WAVE_FORMAT_SONY_SPC",
            TelumAudio => "WAVE_FORMAT_TELUM_AUDIO",
            TelumIaAudio => "WAVE_FORMAT_TELUM_IA_AUDIO",
            NorcomVoiceSystemsAdpcm => "WAVE_FORMAT_NORCOM_VOICE_SYSTEMS_ADPCM",
            FmTownsSnd => "WAVE_FORMAT_FM_TOWNS_SND",
            Micronas => "WAVE_FORMAT_MICRONAS",
            MicronasCelp833 => "WAVE_FORMAT_MICRONAS_CELP833",
            BtvDigital => "WAVE_FORMAT_BTV_DIGITAL",
            IntelMusicCoder => "WAVE_FORMAT_INTEL_MUSIC_CODER",
            IndeoAudio => "WAVE_FORMAT_INDEO_AUDIO",
            QdesignMusic => "WAVE_FORMAT_QDESIGN_MUSIC",
            On2Vp7Audio => "WAVE_FORMAT_ON2_VP7_AUDIO",
            On2Vp6Audio => "WAVE_FORMAT_ON2_VP6_AUDIO",
            VmeVmpcm => "WAVE_FORMAT_VME_VMPCM",
            Tpc => "WAVE_FORMAT_TPC",
            LightwaveLossless => "WAVE_FORMAT_LIGHTWAVE_LOSSLESS",
            Oligsm => "WAVE_FORMAT_OLIGSM",
            Oliadpcm => "WAVE_FORMAT_OLIADPCM",
            Olicelp => "WAVE_FORMAT_OLICELP",
            Olisbc => "WAVE_FORMAT_OLISBC",
            Oliopr => "WAVE_FORMAT_OLIOPR",
            LhCodec => "WAVE_FORMAT_LH_CODEC",
            LhCodecCelp => "WAVE_FORMAT_LH_CODEC_CELP",
            LhCodecSbc8 => "WAVE_FORMAT_LH_CODEC_SBC8",
            LhCodecSbc12 => "WAVE_FORMAT_LH_CODEC_SBC12",
            LhCodecSbc16 => "WAVE_FORMAT_LH_CODEC_SBC16",
            Norris => "WAVE_FORMAT_NORRIS",
            Isiaudio2 => "WAVE_FORMAT_ISIAUDIO_2",
            SoundspaceMusicompress => "WAVE_FORMAT_SOUNDSPACE_MUSICOMPRESS",
            MpegAdtsAac => "WAVE_FORMAT_MPEG_ADTS_AAC",
            MpegRawAac => "WAVE_FORMAT_MPEG_RAW_AAC",
            MpegLoas => "WAVE_FORMAT_MPEG_LOAS",
            NokiaMpegAdtsAac => "WAVE_FORMAT_NOKIA_MPEG_ADTS_AAC",
            NokiaMpegRawAac => "WAVE_FORMAT_NOKIA_MPEG_RAW_AAC",
            VodafoneMpegAdtsAac => "WAVE_FORMAT_VODAFONE_MPEG_ADTS_AAC",
            VodafoneMpegRawAac => "WAVE_FORMAT_VODAFONE_MPEG_RAW_AAC",
            MpegHeaac => "WAVE_FORMAT_MPEG_HEAAC",
            VoxwareRt24Speech => "WAVE_FORMAT_VOXWARE_RT24_SPEECH",
            SonicfoundryLossless => "WAVE_FORMAT_SONICFOUNDRY_LOSSLESS",
            InningsTelecomAdpcm => "WAVE_FORMAT_INNINGS_TELECOM_ADPCM",
            LucentSx8300P => "WAVE_FORMAT_LUCENT_SX8300P",
            LucentSx5363S => "WAVE_FORMAT_LUCENT_SX5363S",
            Cuseeme => "WAVE_FORMAT_CUSEEME",
            NtcsoftAlf2CmAcm => "WAVE_FORMAT_NTCSOFT_ALF2CM_ACM",
            Dvm => "WAVE_FORMAT_DVM",
            Dts2 => "WAVE_FORMAT_DTS2",
            Makeavis => "WAVE_FORMAT_MAKEAVIS",
            DivioMpeg4Aac => "WAVE_FORMAT_DIVIO_MPEG4_AAC",
            NokiaAdaptiveMultirate => "WAVE_FORMAT_NOKIA_ADAPTIVE_MULTIRATE",
            DivioG726 => "WAVE_FORMAT_DIVIO_G726",
            LeadSpeech => "WAVE_FORMAT_LEAD_SPEECH",
            LeadVorbis => "WAVE_FORMAT_LEAD_VORBIS",
            WavpackAudio => "WAVE_FORMAT_WAVPACK_AUDIO",
            Alac => "WAVE_FORMAT_ALAC",
            OggVorbisMode1 => "WAVE_FORMAT_OGG_VORBIS_MODE_1",
            OggVorbisMode2 => "WAVE_FORMAT_OGG_VORBIS_MODE_2",
            OggVorbisMode3 => "WAVE_FORMAT_OGG_VORBIS_MODE_3",
            OggVorbisMode1Plus => "WAVE_FORMAT_OGG_VORBIS_MODE_1_PLUS",
            OggVorbisMode2Plus => "WAVE_FORMAT_OGG_VORBIS_MODE_2_PLUS",
            OggVorbisMode3Plus => "WAVE_FORMAT_OGG_VORBIS_MODE_3_PLUS",
            ThreeComNbx => "WAVE_FORMAT_3COM_NBX",
            Opus => "WAVE_FORMAT_OPUS",
            FaadAac => "WAVE_FORMAT_FAAD_AAC",
            AmrNb => "WAVE_FORMAT_AMR_NB",
            AmrWb => "WAVE_FORMAT_AMR_WB",
            AmrWp => "WAVE_FORMAT_AMR_WP",
            GsmAmrCbr => "WAVE_FORMAT_GSM_AMR_CBR",
            GsmAmrVbrSid => "WAVE_FORMAT_GSM_AMR_VBR_SID",
            ComverseInfosysG7231 => "WAVE_FORMAT_COMVERSE_INFOSYS_G723_1",
            ComverseInfosysAvqsbc => "WAVE_FORMAT_COMVERSE_INFOSYS_AVQSBC",
            ComverseInfosysSbc => "WAVE_FORMAT_COMVERSE_INFOSYS_SBC",
            SymbolG729A => "WAVE_FORMAT_SYMBOL_G729_A",
            VoiceageAmrWb => "WAVE_FORMAT_VOICEAGE_AMR_WB",
            IngenientG726 => "WAVE_FORMAT_INGENIENT_G726",
            Mpeg4Aac => "WAVE_FORMAT_MPEG4_AAC",
            EncoreG726 => "WAVE_FORMAT_ENCORE_G726",
            ZollAsao => "WAVE_FORMAT_ZOLL_ASAO",
            SpeexVoice => "WAVE_FORMAT_SPEEX_VOICE",
            VianixMasc => "WAVE_FORMAT_VIANIX_MASC",
            Wm9SpectrumAnalyzer => "WAVE_FORMAT_WM9_SPECTRUM_ANALYZER",
            WmfSpectrumAnayzer => "WAVE_FORMAT_WMF_SPECTRUM_ANAYZER",
            Gsm610 => "WAVE_FORMAT_GSM_610",
            Gsm620 => "WAVE_FORMAT_GSM_620",
            Gsm660 => "WAVE_FORMAT_GSM_660",
            Gsm690 => "WAVE_FORMAT_GSM_690",
            GsmAdaptiveMultirateWb => "WAVE_FORMAT_GSM_ADAPTIVE_MULTIRATE_WB",
            PolycomG722 => "WAVE_FORMAT_POLYCOM_G722",
            PolycomG728 => "WAVE_FORMAT_POLYCOM_G728",
            PolycomG729A => "WAVE_FORMAT_POLYCOM_G729_A",
            PolycomSiren => "WAVE_FORMAT_POLYCOM_SIREN",
            GlobalIpIlbc => "WAVE_FORMAT_GLOBAL_IP_ILBC",
            RadiotimeTimeShiftRadio => "WAVE_FORMAT_RADIOTIME_TIME_SHIFT_RADIO",
            NiceAca => "WAVE_FORMAT_NICE_ACA",
            NiceAdpcm => "WAVE_FORMAT_NICE_ADPCM",
            VocordG721 => "WAVE_FORMAT_VOCORD_G721",
            VocordG726 => "WAVE_FORMAT_VOCORD_G726",
            VocordG7221 => "WAVE_FORMAT_VOCORD_G722_1",
            VocordG728 => "WAVE_FORMAT_VOCORD_G728",
            VocordG729 => "WAVE_FORMAT_VOCORD_G729",
            VocordG729A => "WAVE_FORMAT_VOCORD_G729_A",
            VocordG7231 => "WAVE_FORMAT_VOCORD_G723_1",
            VocordLbc => "WAVE_FORMAT_VOCORD_LBC",
            NiceG728 => "WAVE_FORMAT_NICE_G728",
            FraceTelecomG729 => "WAVE_FORMAT_FRACE_TELECOM_G729",
            Codian => "WAVE_FORMAT_CODIAN",
            DolbyAc4 => "WAVE_FORMAT_DOLBY_AC4",
            Flac => "WAVE_FORMAT_FLAC",
            Other(_) => "Unknown FormatTag",
            Extensible => "WAVE_FORMAT_EXTENSIBLE",
            Development => "WAVE_FORMAT_DEVELOPMENT",
        };
        write!(f, "{} (0x{:04x})", output, u16::from(*self))?;
        Ok(())
    }
}

impl TryFrom<&FormatTag> for u16 {
    type Error = core::num::TryFromIntError;

    // infalible, but binrw seems to need TryFrom?
    fn try_from(value: &FormatTag) -> Result<Self, Self::Error> {
        Ok(u16::from(*value))
    }
}

/// Provided a consistent interface to the [`FormatTag`] for format variations.
///
/// This may be stored as an enum or const enum variant by the underlying structs.
pub trait Tag {
    /// Return the [`FormatTag`] variant for this struct.
    fn format_tag(&self) -> FormatTag;
}

//---------------------------
/// Format of PCM audio samples in `data`. (WAVE_FORMAT_PCM) [RIFF1991](https://wavref.til.cafe/chunk/fmt/)
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FmtPcm {
    /// A number indicating the WAVE format category of the file.
    #[br(temp, assert(format_tag == Self::FORMAT_TAG))]
    #[bw(calc = Self::FORMAT_TAG)]
    pub format_tag: FormatTag,

    /// The number of channels represented in the waveform data.
    ///
    /// Example: 1 for mono or 2 for stereo.
    pub channels: u16,

    /// The sampling rate at which each channel should be played.
    pub samples_per_sec: u32,

    /// The average number of bytes per second at which the waveform data should be transferred.
    ///
    /// Playback software can estimate the buffer size using this value.
    pub avg_bytes_per_sec: u32,

    /// The block alignment (in bytes) of the waveform data.
    ///
    /// Playback software needs to process a multiple of `block_align` bytes
    /// of data at a time, so the value of `block_align` can be used for
    /// buffer alignment. The `block_align` field should be equal to the
    /// following formula, rounded to the next whole number: `channels` x
    /// ( `bits_per_sample` / 8 )
    pub block_align: u16,

    /// The number of bits used to represent each sample of each channel.
    ///
    /// If there are multiple channels, the sample size is the same for each
    /// channel. The `block_align` field should be equal to the following formula,
    /// rounded to the next whole number: `channels` x ( `bits_per_sample` / 8 )
    pub bits_per_sample: u16,
}

impl KnownChunkID for FmtPcm {
    const ID: FourCC = FourCC(*b"fmt ");
}

impl FmtPcm {
    const FORMAT_TAG: FormatTag = FormatTag::Pcm;
}

impl Tag for FmtPcm {
    fn format_tag(&self) -> FormatTag {
        Self::FORMAT_TAG
    }
}

impl Summarizable for FmtPcm {
    fn summary(&self) -> String {
        format!(
            "{}, {} chan, {}/{}",
            self.format_tag().to_string().replace("WAVE_FORMAT_", ""),
            self.channels,
            self.bits_per_sample,
            self.samples_per_sec,
        )
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }
}

// Iteration based on pattern from https://stackoverflow.com/questions/30218886/how-to-implement-iterator-and-intoiterator-for-a-simple-struct

impl<'a> IntoIterator for &'a FmtPcm {
    type Item = (String, String);
    type IntoIter = FmtPcmIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FmtPcmIterator {
            data: self,
            index: 0,
        }
    }
}

/// Iterate over fields as tuple of Strings (name, value).
#[derive(Debug)]
pub struct FmtPcmIterator<'a> {
    data: &'a FmtPcm,
    index: usize,
}

impl Iterator for FmtPcmIterator<'_> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some(("format_tag".to_string(), self.data.format_tag().to_string())),
            2 => Some(("channels".to_string(), self.data.channels.to_string())),
            3 => Some((
                "samples_per_sec".to_string(),
                self.data.samples_per_sec.to_string(),
            )),
            4 => Some((
                "avg_bytes_per_sec".to_string(),
                self.data.avg_bytes_per_sec.to_string(),
            )),
            5 => Some(("block_align".to_string(), self.data.block_align.to_string())),
            6 => Some((
                "bits_per_sample".to_string(),
                self.data.bits_per_sample.to_string(),
            )),
            _ => None,
        }
    }
}

//---------------------------

/// ADPCM implementation specific decoding coefficients.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AdpcmCoefficients {
    /// ADPCM implementation specific decoding coefficient.
    pub coef1: i16,

    /// ADPCM implementation specific decoding coefficient.
    pub coef2: i16,
}

impl Display for AdpcmCoefficients {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "({}, {})", self.coef1, self.coef2)
    }
}

/// Format of ADPCM audio samples in `data`. (WAVE_FORMAT_ADPCM) [RIFF1994](https://wavref.til.cafe/chunk/fmt/)
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FmtAdpcm {
    /// A number indicating the WAVE format category of the file.
    #[br(temp, assert(format_tag == Self::FORMAT_TAG))]
    #[bw(calc = Self::FORMAT_TAG)]
    pub format_tag: FormatTag,

    /// The number of channels represented in the waveform data.
    ///
    /// Example: 1 for mono or 2 for stereo.
    pub channels: u16,

    /// The sampling rate at which each channel should be played.
    pub samples_per_sec: u32,

    /// The average number of bytes per second at which the waveform data should be transferred.
    ///
    /// Playback software can estimate the buffer size using this value.
    /// ((`samples_per_sec` / `samples_per_block`) * `block_align`).
    pub avg_bytes_per_sec: u32,

    /// The block alignment (in bytes) of the waveform data.
    ///
    /// Playback software needs to process a multiple of `block_align` bytes
    /// of data at a time, so the value of `block_align` can be used for
    /// buffer alignment.
    ///
    /// |(samples_per_sec x channels) | block_align |
    /// |-|-|
    /// |8k | 256|
    /// |11k | 256|
    /// |22k | 512|
    /// |44k | 1024|
    ///
    pub block_align: u16,

    /// The number of bits used to represent each sample of each channel.
    ///
    /// Currently only 4 bits per sample is defined. Other values are reserved.
    pub bits_per_sample: u16,

    /// The count in bytes of the extended data.
    ///
    /// The size in bytes of the extra information in the WAVE format header not
    /// including the size of the [`FmtExtended`] structure. (size of fields from
    /// `format_tag` through `extra_size` inclusive (all fields except `id`, `size` and
    /// the `extra_bytes`))
    #[br()]
    #[bw(map = |_| (self.coefficient_count * 4 + 4) )]
    pub extra_size: u16,

    /// Count of number of samples per block.
    ///
    /// (((`block_align` - (7 * `channels`)) * 8) / (`bits_per_sample` * `channels`)) + 2
    pub samples_per_block: u16,

    /// Count of the number of coefficient sets defined in coefficients.
    pub coefficient_count: u16,

    /// These are the coefficients used by the wave to play.
    ///
    /// They may be interpreted as fixed point 8.8 signed values. Currently
    /// there are 7 preset coefficient sets. They must appear in the following
    /// order.
    ///
    /// |Coef1 |Coef2|
    /// |-|-|
    /// |256 |0|
    /// |512 |-256|
    /// |0 |0|
    /// |192 |64|
    /// |240 |0|
    /// |460 |-208|
    /// |392 |-232|
    ///
    /// Note that if even only 1 coefficient set was used to encode the file then
    /// all coefficient sets are still included. More coefficients may be added
    /// by the encoding software, but the first 7 must always be the same.
    #[br(count = (coefficient_count) as usize)]
    #[bw()]
    pub coefficients: Vec<AdpcmCoefficients>,
}

impl KnownChunkID for FmtAdpcm {
    const ID: FourCC = FourCC(*b"fmt ");
}

impl FmtAdpcm {
    const FORMAT_TAG: FormatTag = FormatTag::Adpcm;
}

impl Tag for FmtAdpcm {
    fn format_tag(&self) -> FormatTag {
        Self::FORMAT_TAG
    }
}

impl Summarizable for FmtAdpcm {
    fn summary(&self) -> String {
        format!(
            "{}, {} chan, {}/{}",
            self.format_tag().to_string().replace("WAVE_FORMAT_", ""),
            self.channels,
            self.bits_per_sample,
            self.samples_per_sec,
        )
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }
}

// Iteration based on pattern from https://stackoverflow.com/questions/30218886/how-to-implement-iterator-and-intoiterator-for-a-simple-struct

impl<'a> IntoIterator for &'a FmtAdpcm {
    type Item = (String, String);
    type IntoIter = FmtAdpcmIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FmtAdpcmIterator {
            data: self,
            index: 0,
        }
    }
}

/// Iterate over fields as tuple of Strings (name, value).
#[derive(Debug)]
pub struct FmtAdpcmIterator<'a> {
    data: &'a FmtAdpcm,
    index: usize,
}

impl Iterator for FmtAdpcmIterator<'_> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some(("format_tag".to_string(), self.data.format_tag().to_string())),
            2 => Some(("channels".to_string(), self.data.channels.to_string())),
            3 => Some((
                "samples_per_sec".to_string(),
                self.data.samples_per_sec.to_string(),
            )),
            4 => Some((
                "avg_bytes_per_sec".to_string(),
                self.data.avg_bytes_per_sec.to_string(),
            )),
            5 => Some(("block_align".to_string(), self.data.block_align.to_string())),
            6 => Some((
                "bits_per_sample".to_string(),
                self.data.bits_per_sample.to_string(),
            )),
            7 => Some(("extra_size".to_string(), self.data.extra_size.to_string())),
            8 => Some((
                "samples_per_block".to_string(),
                self.data.samples_per_block.to_string(),
            )),
            9 => Some((
                "coefficient_count".to_string(),
                self.data.coefficient_count.to_string(),
            )),
            10 => Some((
                "coefficients".to_string(),
                self.data.coefficients.iter().join(" "),
            )),
            _ => None,
        }
    }
}

//---------------------------

/// Format of DVI ADPCM audio samples in `data`. (WAVE_FORMAT_DVI_ADPCM) [RIFF1994](https://wavref.til.cafe/chunk/fmt/)
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FmtDviAdpcm {
    /// A number indicating the WAVE format category of the file.
    #[br(temp, assert(format_tag == Self::FORMAT_TAG))]
    #[bw(calc = Self::FORMAT_TAG)]
    pub format_tag: FormatTag,

    /// The number of channels represented in the waveform data.
    ///
    /// Example: 1 for mono or 2 for stereo.
    pub channels: u16,

    /// Sample rate of the WAVE file. This should be 8000, 11025, 22050 or 44100.
    /// Other sample rates are allowed.
    pub samples_per_sec: u32,

    /// The average number of bytes per second at which the waveform data should be transferred.
    ///
    /// Playback software can estimate the buffer size using this value.
    /// ((`samples_per_sec` / `samples_per_block`) * `block_align`).
    pub avg_bytes_per_sec: u32,

    /// The block alignment (in bytes) of the waveform data.
    ///
    /// Playback software needs to process a multiple of `block_align` bytes
    /// of data at a time, so the value of `block_align` can be used for
    /// buffer alignment.
    ///
    /// |bits_per_sample | block_align |
    /// |-|-|
    /// |3 | (( N * 3 ) + 1 ) * 4 * channels |
    /// |4 | (N + 1) * 4 * channels |
    /// || where N = 0, 1, 2, 3 . . . |    ///
    ///
    /// The recommended block size for coding is 256 * bytes* min(1,
    /// (<`samples_per_second`>/ 11 kHz)) Smaller values cause the block header
    /// to become a more significant storage overhead. But, it is up to the
    /// implementation of the coding portion of the algorithm to decide the
    /// optimal value for `block_align` within the given constraints (see
    /// above). The decoding portion of the algorithm must be able to handle
    /// any valid block size. Playback software needs to process a multiple of
    /// `block_align` bytes of data at a time, so the value of `block_align` can
    /// be used for allocating buffers.
    pub block_align: u16,

    /// The number of bits used to represent each sample of each channel.
    ///
    /// If there are multiple channels, the sample size is the same for each
    /// channel. The `block_align` field should be equal to the following formula,
    /// rounded to the next whole number: `channels` x ( `bits_per_sample` / 8 )
    pub bits_per_sample: u16,

    /// The count in bytes of the extended data.
    ///
    /// The size in bytes of the extra information in the WAVE format header not
    /// including the size of the [`FmtExtended`] structure. (size of fields from
    /// `format_tag` through `extra_size` inclusive (all fields except `id`, `size` and
    /// the `extra_bytes`))
    pub extra_size: u16,

    /// Count of number of samples per block.
    ///
    /// (((`block_align` - (7 * `channels`)) * 8) / (`bits_per_sample` * `channels`)) + 2
    pub samples_per_block: u16,
}

impl KnownChunkID for FmtDviAdpcm {
    const ID: FourCC = FourCC(*b"fmt ");
}

impl FmtDviAdpcm {
    const FORMAT_TAG: FormatTag = FormatTag::DviAdpcm;
}

impl Tag for FmtDviAdpcm {
    fn format_tag(&self) -> FormatTag {
        Self::FORMAT_TAG
    }
}

impl Summarizable for FmtDviAdpcm {
    fn summary(&self) -> String {
        format!(
            "{}, {} chan, {}/{}",
            self.format_tag().to_string().replace("WAVE_FORMAT_", ""),
            self.channels,
            self.bits_per_sample,
            self.samples_per_sec,
        )
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }
}

// Iteration based on pattern from https://stackoverflow.com/questions/30218886/how-to-implement-iterator-and-intoiterator-for-a-simple-struct

impl<'a> IntoIterator for &'a FmtDviAdpcm {
    type Item = (String, String);
    type IntoIter = FmtDviAdpcmIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FmtDviAdpcmIterator {
            data: self,
            index: 0,
        }
    }
}

/// Iterate over fields as tuple of Strings (name, value).
#[derive(Debug)]
pub struct FmtDviAdpcmIterator<'a> {
    data: &'a FmtDviAdpcm,
    index: usize,
}

impl Iterator for FmtDviAdpcmIterator<'_> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some(("format_tag".to_string(), self.data.format_tag().to_string())),
            2 => Some(("channels".to_string(), self.data.channels.to_string())),
            3 => Some((
                "samples_per_sec".to_string(),
                self.data.samples_per_sec.to_string(),
            )),
            4 => Some((
                "avg_bytes_per_sec".to_string(),
                self.data.avg_bytes_per_sec.to_string(),
            )),
            5 => Some(("block_align".to_string(), self.data.block_align.to_string())),
            6 => Some((
                "bits_per_sample".to_string(),
                self.data.bits_per_sample.to_string(),
            )),
            7 => Some(("extra_size".to_string(), self.data.extra_size.to_string())),
            8 => Some((
                "samples_per_block".to_string(),
                self.data.samples_per_block.to_string(),
            )),
            _ => None,
        }
    }
}

//---------------------------
/// `fmt ` Extended format of audio samples in `data`. (WAVEFORMATEX) [RIFF1994](https://wavref.til.cafe/chunk/fmt/)
///
/// This is a fallback parser for unimplmented `FormatTags`, the unparsed
/// extended data is stored in `extra_bytes`.
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FmtExtended {
    // no #[br(assert())] here, this is the default parser
    /// A number indicating the WAVE format category of the file.
    pub format_tag: FormatTag,

    /// The number of channels represented in the waveform data.
    ///
    /// Example: 1 for mono or 2 for stereo.
    pub channels: u16,

    /// The sampling rate at which each channel should be played.
    pub samples_per_sec: u32,

    /// The average number of bytes per second at which the waveform data should be transferred.
    ///
    /// Playback software can estimate the buffer size using this value.
    pub avg_bytes_per_sec: u32,

    /// The block alignment (in bytes) of the waveform data.
    ///
    /// Playback software needs to process a multiple of `block_align` bytes
    /// of data at a time, so the value of `block_align` can be used for
    /// buffer alignment. The `block_align` field should be equal to the
    /// following formula, rounded to the next whole number: `channels` x
    /// ( `bits_per_sample` / 8 )
    pub block_align: u16,

    /// The number of bits used to represent each sample of each channel.
    ///
    /// If there are multiple channels, the sample size is the same for each
    /// channel. The `block_align` field should be equal to the following formula,
    /// rounded to the next whole number: `channels` x ( `bits_per_sample` / 8 )
    pub bits_per_sample: u16,

    /// The count in bytes of the extended data.
    ///
    /// The size in bytes of the extra information in the WAVE format header not
    /// including the size of the `FmtExtended` structure. (size of fields from
    /// format_tag through extra_size inclusive (all fields except id, size and
    /// the extra_bytes))
    #[br()]
    #[bw(map = |_| self.extra_bytes.len() as u16)]
    pub extra_size: u16,

    /// The extra information as bytes.
    ///
    /// `FmtExtended` is intended for use when `format_tag` is unknown, so this
    /// data can't be parsed deteriministically.
    #[br(count = extra_size as usize)]
    #[bw()]
    pub extra_bytes: Vec<u8>,
}

impl KnownChunkID for FmtExtended {
    const ID: FourCC = FourCC(*b"fmt ");
}

impl Tag for FmtExtended {
    fn format_tag(&self) -> FormatTag {
        self.format_tag
    }
}

impl Summarizable for FmtExtended {
    fn summary(&self) -> String {
        format!(
            "{}, {} chan, {}/{}, EX: {}",
            self.format_tag.to_string().replace("WAVE_FORMAT_", ""),
            self.channels,
            self.bits_per_sample,
            self.samples_per_sec,
            self.extra_size,
        )
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }

    fn item_summary_header(&self) -> String {
        "(parsed as WAVEFORMATEX; this format tag not implemented)".to_string()
    }
}

// Iteration based on pattern from https://stackoverflow.com/questions/30218886/how-to-implement-iterator-and-intoiterator-for-a-simple-struct

impl<'a> IntoIterator for &'a FmtExtended {
    type Item = (String, String);
    type IntoIter = FmtExtendedIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FmtExtendedIterator {
            data: self,
            index: 0,
        }
    }
}

/// Iterate over fields as tuple of Strings (name, value).
#[derive(Debug)]
pub struct FmtExtendedIterator<'a> {
    data: &'a FmtExtended,
    index: usize,
}

impl Iterator for FmtExtendedIterator<'_> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some(("format_tag".to_string(), self.data.format_tag.to_string())),
            2 => Some(("channels".to_string(), self.data.channels.to_string())),
            3 => Some((
                "samples_per_sec".to_string(),
                self.data.samples_per_sec.to_string(),
            )),
            4 => Some((
                "avg_bytes_per_sec".to_string(),
                self.data.avg_bytes_per_sec.to_string(),
            )),
            5 => Some(("block_align".to_string(), self.data.block_align.to_string())),
            6 => Some((
                "bits_per_sample".to_string(),
                self.data.bits_per_sample.to_string(),
            )),
            7 => Some(("extra_size".to_string(), self.data.extra_size.to_string())),
            8 => Some((
                "extra_bytes".to_string(),
                format!("0x{}", hex::encode_upper(&self.data.extra_bytes)),
            )),
            _ => None,
        }
    }
}

/// All `Fmt` structs as an enum.
///
/// TODO: document design of Fmt* structs.
#[allow(missing_docs)]
#[binrw]
#[brw(little)]
#[br(import(_size: u32))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FmtEnum {
    Pcm(FmtPcm),
    Adpcm(FmtAdpcm),
    DviAdpcm(FmtDviAdpcm),
    Extended(FmtExtended),
}

impl KnownChunkID for FmtEnum {
    const ID: FourCC = FourCC(*b"fmt ");
}

impl Tag for FmtEnum {
    fn format_tag(&self) -> FormatTag {
        match self {
            FmtEnum::Pcm(e) => e.format_tag(),
            FmtEnum::Adpcm(e) => e.format_tag(),
            FmtEnum::DviAdpcm(e) => e.format_tag(),
            FmtEnum::Extended(e) => e.format_tag(),
        }
    }
}

impl Summarizable for FmtEnum {
    fn summary(&self) -> String {
        match self {
            FmtEnum::Pcm(e) => e.summary(),
            FmtEnum::Adpcm(e) => e.summary(),
            FmtEnum::DviAdpcm(e) => e.summary(),
            FmtEnum::Extended(e) => e.summary(),
        }
    }

    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        match self {
            FmtEnum::Pcm(e) => e.items(),
            FmtEnum::Adpcm(e) => e.items(),
            FmtEnum::DviAdpcm(e) => e.items(),
            FmtEnum::Extended(e) => e.items(),
        }
    }

    fn name(&self) -> String {
        match self {
            FmtEnum::Pcm(e) => e.name(),
            FmtEnum::Adpcm(e) => e.name(),
            FmtEnum::DviAdpcm(e) => e.name(),
            FmtEnum::Extended(e) => e.name(),
        }
    }

    fn item_summary_header(&self) -> String {
        match self {
            FmtEnum::Pcm(e) => e.item_summary_header(),
            FmtEnum::Adpcm(e) => e.item_summary_header(),
            FmtEnum::DviAdpcm(e) => e.item_summary_header(),
            FmtEnum::Extended(e) => e.item_summary_header(),
        }
    }
}

/// `fmt ` Format of audio samples in `data`. [RIFF1991](https://wavref.til.cafe/chunk/fmt/), [RIFF1994](https://wavref.til.cafe/chunk/fmt/)
pub type FmtChunk = KnownChunk<FmtEnum>;

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {
    use binrw::BinRead;

    use super::*;
    use crate::testing::hex_to_cursor;

    #[test]
    fn parse_fmt() {
        let mut buff = hex_to_cursor("666D7420 10000000 01000100 80BB0000 80320200 03001800");
        let expected = FmtChunk {
            offset: Some(0),
            size: 16,
            data: FmtEnum::Pcm(FmtPcm {
                channels: 1,
                samples_per_sec: 48000,
                avg_bytes_per_sec: 144000,
                block_align: 3,
                bits_per_sample: 24,
            }),
            extra_bytes: vec![],
        };
        let chunk = FmtChunk::read(&mut buff).expect("error parsing WAV chunks");
        assert_eq!(chunk, expected);

        // make sure parsing via SizedChunkEnum also works
        use crate::SizedChunkEnum;
        buff.set_position(0);
        let chunk = SizedChunkEnum::read(&mut buff).expect("error parsing WAV chunks");
        assert_eq!(chunk, SizedChunkEnum::Fmt(expected));
    }

    #[test]
    fn formattag_primitive() {
        let pcm = FormatTag::from(1_u16);
        assert_eq!(pcm, FormatTag::Pcm);

        // make sure parsing into Other() works
        let unknown = FormatTag::from(0x4242_u16);
        assert_eq!(unknown, FormatTag::Other(0x4242_u16));
        assert_eq!(unknown.to_string(), "Unknown FormatTag (0x4242)");
    }

    #[test]
    fn parse_fmt_adpcm() {
        let expected = FormatTag::Adpcm;

        let mut buff = hex_to_cursor(
            "666D7420 32000000 02000100 80BB0000 4D5E0000 00040400 2000F407 07000001 00000002 00FF0000 0000C000 4000F000 0000CC01 30FF8801 18FF",
        );
        let chunk = FmtChunk::read(&mut buff).expect("error parsing WAV chunks");
        if let FmtEnum::Adpcm(fmt) = chunk.data {
            assert_eq!(fmt.format_tag(), expected);
            assert_eq!(fmt.channels, 1);
            assert_eq!(fmt.extra_size, 32);
            assert_eq!(fmt.samples_per_block, 2036);
            assert_eq!(fmt.coefficient_count, 7);
        } else {
            panic!(
                "variant match failed, expected FmtEnum::Adpcm, got: {:?}",
                chunk
            );
        }
    }

    #[test]
    fn parse_fmt_dvi_adpcm() {
        let expected = FormatTag::DviAdpcm;

        let mut buff =
            hex_to_cursor("666D7420 14000000 11000200 44AC0000 DBAC0000 00080400 0200F907");
        let chunk = FmtChunk::read(&mut buff).expect("error parsing WAV chunks");
        if let FmtEnum::DviAdpcm(fmt) = chunk.data {
            assert_eq!(fmt.format_tag(), expected);
            assert_eq!(fmt.extra_size, 2);
            assert_eq!(fmt.samples_per_block, 2041);
        } else {
            panic!(
                "variant match failed, expected FmtEnum::DviAdpcm, got: {:?}",
                chunk
            );
        }
    }

    #[test]
    fn parse_fmt_extended() {
        let expected = FormatTag::Unknown;

        let mut buff = hex_to_cursor(
            "666D7420 14000000 00000200 44AC0000
        DBAC0000 00080400 0200F907",
        );
        let chunk = FmtChunk::read(&mut buff).expect("error parsing WAV chunks");
        if let FmtEnum::Extended(fmt) = chunk.data {
            assert_eq!(fmt.format_tag, expected);
        } else {
            unreachable!("variant match failed, FmtExtended should cover all cases");
        }
    }

    // compile time check to ensure all chunks implement consistent traits
    fn has_fmt_standard_traits<T>()
    where
        T: Tag,
    {
    }

    #[test]
    fn consistent_traits() {
        // this Enum transitively ensures the traits of all subchunks
        has_fmt_standard_traits::<FmtEnum>();
    }
}
