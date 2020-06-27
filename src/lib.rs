/*
Copyright 2017 William Cody Laeder

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

//! Intel RTM Extensions.
//!
//! Please note this crate only works on x86_64 Intel
//! processors, and only those built after
//! the boardwell 6th generation.
//!
//!# Basic Intro:
//!
//! RTM works very similiar to a database. You can
//! read/write memory but you have to commit the
//! changes. If another thread modifies the same
//! region as you are, the other RTM transaction
//! will abort (the second chronologically).
//!
//! RTM transaction can also be cancelled. Meaning
//! if you do not want to commit a transaction
//! as in you wish to roll it back that can be
//! accomplished via `abort(x: u8)` interface
//! within this library if you hit a condition
//! that requires rolling back the transaction.
//!
//!# Deep Dive:
//!
//! Now we need to perform a deep dive into
//! into RTM and it's implementation. RTM works on
//! the cache line level. This means each region
//! RTM _thinks_ it is exclusive to a cache line.
//! Each cache line in Intel CPU's is 64bytes,
//! so you will wish to ensure that your data
//! structures being modified WITHIN RTM
//! transactions are `X * 64 = size_of::<T>()`
//! or `0 == size_of::<T>() % 64`. At the same
//! time you will wish to ensure the allocation
//! is on the 64 byte boundry (this is called
//! allignment) this simply means
//! `  &T % 64 == 0` (the physical pointer).
//!
//! The reason for this false sharing. If a
//! different thread modifies the same cacheline
//! you have decared RTM your modification may
//! abort reducing your preformance.
//!
//! RTM works via the [MESIF](https://en.wikipedia.org/wiki/MESIF_protocol) protocol. These are
//! the states a Cache Line can be in. E (Exclusive),
//! M (Modified), S (Shared), F (Forward), I (Invalid).
//! Effectively RTM attempts to ensure that all the
//! writes/reads you will perform are on E/F values
//! (Exclusive/Forward). This means you either own the
//! the only copy of this in Cache OR another thread may
//! read this data, but not write to it.
//!
//! If another thread attempts to write to a cacheline
//! during the RTM transaction the status of your cache
//! will change `E -> S` or `F -> I`. And the other
//! thread is not executing RTM code, your transaction
//! will abort.
//!
//!# Architecture Notes:
//!
//! RTM changes are buffered in L1 cache.
//! so too many changes can result in very extreme
//! performance penalities.
//!
//! RMT changes are a full instruction barrier, but
//! they are not the same as an `mfence` or `sfence`
//! or `lfence` instruction (only to the local cache
//! lines effected by an RTM transaction).
//!
//!# Performance Notes:
//!
//! For modification of a single cache line
//! `AtomicUsize` or `AtomicPtr` will be faster even
//! in `SeqCst` mode. RTM transaction are typically
//! faster for larger transaction on the order of
//! several cache lines (typically `>300` bytes) or so.
//!

#![allow(non_upper_case_globals)]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(stdsimd)]

/// This function performs a transaction. If the transaction fails
/// or is aborted, it returns the correct error.
#[cfg(all(target_feature = "rtm", target_arch = "x86_64"))]
#[allow(dead_code)]
pub fn transaction<S, F>(data: &mut S, lambda: F) -> Result<(), AbortCode>
where
    S: Sync,
    F: FnOnce(&mut S),
{
    //aadfadafsf();
    match unsafe { crate::tsx::_xbegin() } {
        0xFFFFFFFF => {
            lambda(data);
            unsafe { crate::tsx::_xend() };
            Ok(())
        }
        arg => into_abort(arg),
    }
}

/// Unlike `transaction` this function can perform retries.
///
/// If you would like to disable that action pass `None` to
/// the argument.
///
/// Otherwise the `usize` value passed will be assumed the
/// number of retries to make.
///
/// Any abort code other than `retry` will be returned.
#[cfg(all(target_feature = "rtm", target_arch = "x86_64"))]
#[allow(dead_code)]
pub fn transaction_retry<S, F, R>(data: &mut S, lambda: F, retries: R) -> Result<(), AbortCode>
where
    S: Sync,
    F: Fn(&mut S),
    R: Into<Option<usize>>,
{
    let retries = match retries.into() {
        Option::None => 0,
        Option::Some(x) => x,
    };
    let mut curr = 0usize;
    loop {
        match crate::transaction(data, &lambda) {
            Err(AbortCode::Retry) => {
                curr += 1;
                if curr >= retries {
                    return Err(AbortCode::Retry);
                }
                continue;
            }
            output => return output,
        };
    }
}

/// aborts the transaction if one is present
#[cfg(all(target_arch = "x86_64", target_feature = "rtm"))]
pub fn abort(code: u8) {
    match code {
        00 => crate::abort_functions::abort_0(),
        01 => crate::abort_functions::abort_1(),
        02 => crate::abort_functions::abort_2(),
        03 => crate::abort_functions::abort_3(),
        04 => crate::abort_functions::abort_4(),
        05 => crate::abort_functions::abort_5(),
        06 => crate::abort_functions::abort_6(),
        07 => crate::abort_functions::abort_7(),
        08 => crate::abort_functions::abort_8(),
        09 => crate::abort_functions::abort_9(),

        10 => crate::abort_functions::abort_10(),
        11 => crate::abort_functions::abort_11(),
        12 => crate::abort_functions::abort_12(),
        13 => crate::abort_functions::abort_13(),
        14 => crate::abort_functions::abort_14(),
        15 => crate::abort_functions::abort_15(),
        16 => crate::abort_functions::abort_16(),
        17 => crate::abort_functions::abort_17(),
        18 => crate::abort_functions::abort_18(),
        19 => crate::abort_functions::abort_19(),

        20 => crate::abort_functions::abort_20(),
        21 => crate::abort_functions::abort_21(),
        22 => crate::abort_functions::abort_22(),
        23 => crate::abort_functions::abort_23(),
        24 => crate::abort_functions::abort_24(),
        25 => crate::abort_functions::abort_25(),
        26 => crate::abort_functions::abort_26(),
        27 => crate::abort_functions::abort_27(),
        28 => crate::abort_functions::abort_28(),
        29 => crate::abort_functions::abort_29(),

        30 => crate::abort_functions::abort_30(),
        31 => crate::abort_functions::abort_31(),
        32 => crate::abort_functions::abort_32(),
        33 => crate::abort_functions::abort_33(),
        34 => crate::abort_functions::abort_34(),
        35 => crate::abort_functions::abort_35(),
        36 => crate::abort_functions::abort_36(),
        37 => crate::abort_functions::abort_37(),
        38 => crate::abort_functions::abort_38(),
        39 => crate::abort_functions::abort_39(),

        40 => crate::abort_functions::abort_40(),
        41 => crate::abort_functions::abort_41(),
        42 => crate::abort_functions::abort_42(),
        43 => crate::abort_functions::abort_43(),
        44 => crate::abort_functions::abort_44(),
        45 => crate::abort_functions::abort_45(),
        46 => crate::abort_functions::abort_46(),
        47 => crate::abort_functions::abort_47(),
        48 => crate::abort_functions::abort_48(),
        49 => crate::abort_functions::abort_49(),

        50 => crate::abort_functions::abort_50(),
        51 => crate::abort_functions::abort_51(),
        52 => crate::abort_functions::abort_52(),
        53 => crate::abort_functions::abort_53(),
        54 => crate::abort_functions::abort_54(),
        55 => crate::abort_functions::abort_55(),
        56 => crate::abort_functions::abort_56(),
        57 => crate::abort_functions::abort_57(),
        58 => crate::abort_functions::abort_58(),
        59 => crate::abort_functions::abort_59(),

        60 => crate::abort_functions::abort_60(),
        61 => crate::abort_functions::abort_61(),
        62 => crate::abort_functions::abort_62(),
        63 => crate::abort_functions::abort_63(),
        64 => crate::abort_functions::abort_64(),
        65 => crate::abort_functions::abort_65(),
        66 => crate::abort_functions::abort_66(),
        67 => crate::abort_functions::abort_67(),
        68 => crate::abort_functions::abort_68(),
        69 => crate::abort_functions::abort_69(),

        70 => crate::abort_functions::abort_70(),
        71 => crate::abort_functions::abort_71(),
        72 => crate::abort_functions::abort_72(),
        73 => crate::abort_functions::abort_73(),
        74 => crate::abort_functions::abort_74(),
        75 => crate::abort_functions::abort_75(),
        76 => crate::abort_functions::abort_76(),
        77 => crate::abort_functions::abort_77(),
        78 => crate::abort_functions::abort_78(),
        79 => crate::abort_functions::abort_79(),

        80 => crate::abort_functions::abort_80(),
        81 => crate::abort_functions::abort_81(),
        82 => crate::abort_functions::abort_82(),
        83 => crate::abort_functions::abort_83(),
        84 => crate::abort_functions::abort_84(),
        85 => crate::abort_functions::abort_85(),
        86 => crate::abort_functions::abort_86(),
        87 => crate::abort_functions::abort_87(),
        88 => crate::abort_functions::abort_88(),
        89 => crate::abort_functions::abort_89(),

        90 => crate::abort_functions::abort_90(),
        91 => crate::abort_functions::abort_91(),
        92 => crate::abort_functions::abort_92(),
        93 => crate::abort_functions::abort_93(),
        94 => crate::abort_functions::abort_94(),
        95 => crate::abort_functions::abort_95(),
        96 => crate::abort_functions::abort_96(),
        97 => crate::abort_functions::abort_97(),
        98 => crate::abort_functions::abort_98(),
        99 => crate::abort_functions::abort_99(),

        100 => crate::abort_functions::abort_100(),
        101 => crate::abort_functions::abort_101(),
        102 => crate::abort_functions::abort_102(),
        103 => crate::abort_functions::abort_103(),
        104 => crate::abort_functions::abort_104(),
        105 => crate::abort_functions::abort_105(),
        106 => crate::abort_functions::abort_106(),
        107 => crate::abort_functions::abort_107(),
        108 => crate::abort_functions::abort_108(),
        109 => crate::abort_functions::abort_109(),

        110 => crate::abort_functions::abort_110(),
        111 => crate::abort_functions::abort_111(),
        112 => crate::abort_functions::abort_112(),
        113 => crate::abort_functions::abort_113(),
        114 => crate::abort_functions::abort_114(),
        115 => crate::abort_functions::abort_115(),
        116 => crate::abort_functions::abort_116(),
        117 => crate::abort_functions::abort_117(),
        118 => crate::abort_functions::abort_118(),
        119 => crate::abort_functions::abort_119(),

        120 => crate::abort_functions::abort_120(),
        121 => crate::abort_functions::abort_121(),
        122 => crate::abort_functions::abort_122(),
        123 => crate::abort_functions::abort_123(),
        124 => crate::abort_functions::abort_124(),
        125 => crate::abort_functions::abort_125(),
        126 => crate::abort_functions::abort_126(),
        127 => crate::abort_functions::abort_127(),
        128 => crate::abort_functions::abort_128(),
        129 => crate::abort_functions::abort_129(),

        130 => crate::abort_functions::abort_130(),
        131 => crate::abort_functions::abort_131(),
        132 => crate::abort_functions::abort_132(),
        133 => crate::abort_functions::abort_133(),
        134 => crate::abort_functions::abort_134(),
        135 => crate::abort_functions::abort_135(),
        136 => crate::abort_functions::abort_136(),
        137 => crate::abort_functions::abort_137(),
        138 => crate::abort_functions::abort_138(),
        139 => crate::abort_functions::abort_139(),

        140 => crate::abort_functions::abort_140(),
        141 => crate::abort_functions::abort_141(),
        142 => crate::abort_functions::abort_142(),
        143 => crate::abort_functions::abort_143(),
        144 => crate::abort_functions::abort_144(),
        145 => crate::abort_functions::abort_145(),
        146 => crate::abort_functions::abort_146(),
        147 => crate::abort_functions::abort_147(),
        148 => crate::abort_functions::abort_148(),
        149 => crate::abort_functions::abort_149(),

        150 => crate::abort_functions::abort_150(),
        151 => crate::abort_functions::abort_151(),
        152 => crate::abort_functions::abort_152(),
        153 => crate::abort_functions::abort_153(),
        154 => crate::abort_functions::abort_154(),
        155 => crate::abort_functions::abort_155(),
        156 => crate::abort_functions::abort_156(),
        157 => crate::abort_functions::abort_157(),
        158 => crate::abort_functions::abort_158(),
        159 => crate::abort_functions::abort_159(),

        160 => crate::abort_functions::abort_160(),
        161 => crate::abort_functions::abort_161(),
        162 => crate::abort_functions::abort_162(),
        163 => crate::abort_functions::abort_163(),
        164 => crate::abort_functions::abort_164(),
        165 => crate::abort_functions::abort_165(),
        166 => crate::abort_functions::abort_166(),
        167 => crate::abort_functions::abort_167(),
        168 => crate::abort_functions::abort_168(),
        169 => crate::abort_functions::abort_169(),

        170 => crate::abort_functions::abort_170(),
        171 => crate::abort_functions::abort_171(),
        172 => crate::abort_functions::abort_172(),
        173 => crate::abort_functions::abort_173(),
        174 => crate::abort_functions::abort_174(),
        175 => crate::abort_functions::abort_175(),
        176 => crate::abort_functions::abort_176(),
        177 => crate::abort_functions::abort_177(),
        178 => crate::abort_functions::abort_178(),
        179 => crate::abort_functions::abort_179(),

        180 => crate::abort_functions::abort_180(),
        181 => crate::abort_functions::abort_181(),
        182 => crate::abort_functions::abort_182(),
        183 => crate::abort_functions::abort_183(),
        184 => crate::abort_functions::abort_184(),
        185 => crate::abort_functions::abort_185(),
        186 => crate::abort_functions::abort_186(),
        187 => crate::abort_functions::abort_187(),
        188 => crate::abort_functions::abort_188(),
        189 => crate::abort_functions::abort_189(),

        190 => crate::abort_functions::abort_190(),
        191 => crate::abort_functions::abort_191(),
        192 => crate::abort_functions::abort_192(),
        193 => crate::abort_functions::abort_193(),
        194 => crate::abort_functions::abort_194(),
        195 => crate::abort_functions::abort_195(),
        196 => crate::abort_functions::abort_196(),
        197 => crate::abort_functions::abort_197(),
        198 => crate::abort_functions::abort_198(),
        199 => crate::abort_functions::abort_199(),

        200 => crate::abort_functions::abort_200(),
        201 => crate::abort_functions::abort_201(),
        202 => crate::abort_functions::abort_202(),
        203 => crate::abort_functions::abort_203(),
        204 => crate::abort_functions::abort_204(),
        205 => crate::abort_functions::abort_205(),
        206 => crate::abort_functions::abort_206(),
        207 => crate::abort_functions::abort_207(),
        208 => crate::abort_functions::abort_208(),
        209 => crate::abort_functions::abort_209(),

        210 => crate::abort_functions::abort_210(),
        211 => crate::abort_functions::abort_211(),
        212 => crate::abort_functions::abort_212(),
        213 => crate::abort_functions::abort_213(),
        214 => crate::abort_functions::abort_214(),
        215 => crate::abort_functions::abort_215(),
        216 => crate::abort_functions::abort_216(),
        217 => crate::abort_functions::abort_217(),
        218 => crate::abort_functions::abort_218(),
        219 => crate::abort_functions::abort_219(),

        220 => crate::abort_functions::abort_220(),
        221 => crate::abort_functions::abort_221(),
        222 => crate::abort_functions::abort_222(),
        223 => crate::abort_functions::abort_223(),
        224 => crate::abort_functions::abort_224(),
        225 => crate::abort_functions::abort_225(),
        226 => crate::abort_functions::abort_226(),
        227 => crate::abort_functions::abort_227(),
        228 => crate::abort_functions::abort_228(),
        229 => crate::abort_functions::abort_229(),

        230 => crate::abort_functions::abort_230(),
        231 => crate::abort_functions::abort_231(),
        232 => crate::abort_functions::abort_232(),
        233 => crate::abort_functions::abort_233(),
        234 => crate::abort_functions::abort_234(),
        235 => crate::abort_functions::abort_235(),
        236 => crate::abort_functions::abort_236(),
        237 => crate::abort_functions::abort_237(),
        238 => crate::abort_functions::abort_238(),
        239 => crate::abort_functions::abort_239(),

        240 => crate::abort_functions::abort_240(),
        241 => crate::abort_functions::abort_241(),
        242 => crate::abort_functions::abort_242(),
        243 => crate::abort_functions::abort_243(),
        244 => crate::abort_functions::abort_244(),
        245 => crate::abort_functions::abort_245(),
        246 => crate::abort_functions::abort_246(),
        247 => crate::abort_functions::abort_247(),
        248 => crate::abort_functions::abort_248(),
        249 => crate::abort_functions::abort_249(),

        250 => crate::abort_functions::abort_250(),
        251 => crate::abort_functions::abort_251(),
        252 => crate::abort_functions::abort_252(),
        253 => crate::abort_functions::abort_253(),
        254 => crate::abort_functions::abort_254(),
        255 => crate::abort_functions::abort_255(),
    }
}

/// This contains the various abort functions.
///
/// The value has to be constant, so each code
/// is broken into its own function. Less than
/// ideal, but workable.
mod abort_functions {

    macro_rules! abort_codes {
        ($( $name: ident => $code: expr),* $(,)*) => {
            $(
                #[cfg(all(target_arch="x86_64", target_feature="rtm"))]
                pub fn $name() {
                    unsafe {

                        #[cfg(feature="std")]
                        {
                            if !is_x86_feature_detected!("rtm") {
                                panic!("rtm not detected");
                            }
                        }

                        if crate::tsx::_xtest() != 0 {
                            return;
                        }
                        crate::tsx::_xabort($code);
                    }
                }
            )*
        }
    }

    abort_codes! {
        abort_0 => 00,
        abort_1 => 01,
        abort_2 => 02,
        abort_3 => 03,
        abort_4 => 04,
        abort_5 => 05,
        abort_6 => 06,
        abort_7 => 07,
        abort_8 => 08,
        abort_9 => 09,

        abort_10 => 10,
        abort_11 => 11,
        abort_12 => 12,
        abort_13 => 13,
        abort_14 => 14,
        abort_15 => 15,
        abort_16 => 16,
        abort_17 => 17,
        abort_18 => 18,
        abort_19 => 19,

        abort_20 => 20,
        abort_21 => 21,
        abort_22 => 22,
        abort_23 => 23,
        abort_24 => 24,
        abort_25 => 25,
        abort_26 => 26,
        abort_27 => 27,
        abort_28 => 28,
        abort_29 => 29,

        abort_30 => 30,
        abort_31 => 31,
        abort_32 => 32,
        abort_33 => 33,
        abort_34 => 34,
        abort_35 => 35,
        abort_36 => 36,
        abort_37 => 37,
        abort_38 => 38,
        abort_39 => 39,

        abort_40 => 40,
        abort_41 => 41,
        abort_42 => 42,
        abort_43 => 43,
        abort_44 => 44,
        abort_45 => 45,
        abort_46 => 46,
        abort_47 => 47,
        abort_48 => 48,
        abort_49 => 49,

        abort_50 => 50,
        abort_51 => 51,
        abort_52 => 52,
        abort_53 => 53,
        abort_54 => 54,
        abort_55 => 55,
        abort_56 => 56,
        abort_57 => 57,
        abort_58 => 58,
        abort_59 => 59,

        abort_60 => 60,
        abort_61 => 61,
        abort_62 => 62,
        abort_63 => 63,
        abort_64 => 64,
        abort_65 => 65,
        abort_66 => 66,
        abort_67 => 67,
        abort_68 => 68,
        abort_69 => 69,

        abort_70 => 70,
        abort_71 => 71,
        abort_72 => 72,
        abort_73 => 73,
        abort_74 => 74,
        abort_75 => 75,
        abort_76 => 76,
        abort_77 => 77,
        abort_78 => 78,
        abort_79 => 79,

        abort_80 => 80,
        abort_81 => 81,
        abort_82 => 82,
        abort_83 => 83,
        abort_84 => 84,
        abort_85 => 85,
        abort_86 => 86,
        abort_87 => 87,
        abort_88 => 88,
        abort_89 => 89,

        abort_90 => 90,
        abort_91 => 91,
        abort_92 => 92,
        abort_93 => 93,
        abort_94 => 94,
        abort_95 => 95,
        abort_96 => 96,
        abort_97 => 97,
        abort_98 => 98,
        abort_99 => 99,

        abort_100 => 100,
        abort_101 => 101,
        abort_102 => 102,
        abort_103 => 103,
        abort_104 => 104,
        abort_105 => 105,
        abort_106 => 106,
        abort_107 => 107,
        abort_108 => 108,
        abort_109 => 109,

        abort_110 => 110,
        abort_111 => 111,
        abort_112 => 112,
        abort_113 => 113,
        abort_114 => 114,
        abort_115 => 115,
        abort_116 => 116,
        abort_117 => 117,
        abort_118 => 118,
        abort_119 => 119,

        abort_120 => 120,
        abort_121 => 121,
        abort_122 => 122,
        abort_123 => 123,
        abort_124 => 124,
        abort_125 => 125,
        abort_126 => 126,
        abort_127 => 127,
        abort_128 => 128,
        abort_129 => 129,

        abort_130 => 130,
        abort_131 => 131,
        abort_132 => 132,
        abort_133 => 133,
        abort_134 => 134,
        abort_135 => 135,
        abort_136 => 136,
        abort_137 => 137,
        abort_138 => 138,
        abort_139 => 139,

        abort_140 => 140,
        abort_141 => 141,
        abort_142 => 142,
        abort_143 => 143,
        abort_144 => 144,
        abort_145 => 145,
        abort_146 => 146,
        abort_147 => 147,
        abort_148 => 148,
        abort_149 => 149,

        abort_150 => 150,
        abort_151 => 151,
        abort_152 => 152,
        abort_153 => 153,
        abort_154 => 154,
        abort_155 => 155,
        abort_156 => 156,
        abort_157 => 157,
        abort_158 => 158,
        abort_159 => 159,

        abort_160 => 160,
        abort_161 => 161,
        abort_162 => 162,
        abort_163 => 163,
        abort_164 => 164,
        abort_165 => 165,
        abort_166 => 166,
        abort_167 => 167,
        abort_168 => 168,
        abort_169 => 169,

        abort_170 => 170,
        abort_171 => 171,
        abort_172 => 172,
        abort_173 => 173,
        abort_174 => 174,
        abort_175 => 175,
        abort_176 => 176,
        abort_177 => 177,
        abort_178 => 178,
        abort_179 => 179,

        abort_180 => 180,
        abort_181 => 181,
        abort_182 => 182,
        abort_183 => 183,
        abort_184 => 184,
        abort_185 => 185,
        abort_186 => 186,
        abort_187 => 187,
        abort_188 => 188,
        abort_189 => 189,

        abort_190 => 190,
        abort_191 => 191,
        abort_192 => 192,
        abort_193 => 193,
        abort_194 => 194,
        abort_195 => 195,
        abort_196 => 196,
        abort_197 => 197,
        abort_198 => 198,
        abort_199 => 199,

        abort_200 => 200,
        abort_201 => 201,
        abort_202 => 202,
        abort_203 => 203,
        abort_204 => 204,
        abort_205 => 205,
        abort_206 => 206,
        abort_207 => 207,
        abort_208 => 208,
        abort_209 => 209,

        abort_210 => 210,
        abort_211 => 211,
        abort_212 => 212,
        abort_213 => 213,
        abort_214 => 214,
        abort_215 => 215,
        abort_216 => 216,
        abort_217 => 217,
        abort_218 => 218,
        abort_219 => 219,

        abort_220 => 220,
        abort_221 => 221,
        abort_222 => 222,
        abort_223 => 223,
        abort_224 => 224,
        abort_225 => 225,
        abort_226 => 226,
        abort_227 => 227,
        abort_228 => 228,
        abort_229 => 229,

        abort_230 => 230,
        abort_231 => 231,
        abort_232 => 232,
        abort_233 => 233,
        abort_234 => 234,
        abort_235 => 235,
        abort_236 => 236,
        abort_237 => 237,
        abort_238 => 238,
        abort_239 => 239,

        abort_240 => 240,
        abort_241 => 241,
        abort_242 => 242,
        abort_243 => 243,
        abort_244 => 244,
        abort_245 => 245,
        abort_246 => 246,
        abort_247 => 247,
        abort_248 => 248,
        abort_249 => 249,

        abort_250 => 250,
        abort_251 => 251,
        abort_252 => 252,
        abort_253 => 253,
        abort_254 => 254,
        abort_255 => 255,
    }
}

mod abort_codes {

    macro_rules! code_gen {
        ($($name: ident => $value: expr),* $(,)*) => {
            $(
                #[allow(dead_code)]
                pub const $name: u32 = ($value << 24) + 1;
            )*
        }
    }
    code_gen! {
        abort_0 => 00u32,
        abort_1 => 01u32,
        abort_2 => 02u32,
        abort_3 => 03u32,
        abort_4 => 04u32,
        abort_5 => 05u32,
        abort_6 => 06u32,
        abort_7 => 07u32,
        abort_8 => 08u32,
        abort_9 => 09u32,

        abort_10 => 10u32,
        abort_11 => 11u32,
        abort_12 => 12u32,
        abort_13 => 13u32,
        abort_14 => 14u32,
        abort_15 => 15u32,
        abort_16 => 16u32,
        abort_17 => 17u32,
        abort_18 => 18u32,
        abort_19 => 19u32,

        abort_20 => 20u32,
        abort_21 => 21u32,
        abort_22 => 22u32,
        abort_23 => 23u32,
        abort_24 => 24u32,
        abort_25 => 25u32,
        abort_26 => 26u32,
        abort_27 => 27u32,
        abort_28 => 28u32,
        abort_29 => 29u32,

        abort_30 => 30u32,
        abort_31 => 31u32,
        abort_32 => 32u32,
        abort_33 => 33u32,
        abort_34 => 34u32,
        abort_35 => 35u32,
        abort_36 => 36u32,
        abort_37 => 37u32,
        abort_38 => 38u32,
        abort_39 => 39u32,

        abort_40 => 40u32,
        abort_41 => 41u32,
        abort_42 => 42u32,
        abort_43 => 43u32,
        abort_44 => 44u32,
        abort_45 => 45u32,
        abort_46 => 46u32,
        abort_47 => 47u32,
        abort_48 => 48u32,
        abort_49 => 49u32,

        abort_50 => 50u32,
        abort_51 => 51u32,
        abort_52 => 52u32,
        abort_53 => 53u32,
        abort_54 => 54u32,
        abort_55 => 55u32,
        abort_56 => 56u32,
        abort_57 => 57u32,
        abort_58 => 58u32,
        abort_59 => 59u32,

        abort_60 => 60u32,
        abort_61 => 61u32,
        abort_62 => 62u32,
        abort_63 => 63u32,
        abort_64 => 64u32,
        abort_65 => 65u32,
        abort_66 => 66u32,
        abort_67 => 67u32,
        abort_68 => 68u32,
        abort_69 => 69u32,

        abort_70 => 70u32,
        abort_71 => 71u32,
        abort_72 => 72u32,
        abort_73 => 73u32,
        abort_74 => 74u32,
        abort_75 => 75u32,
        abort_76 => 76u32,
        abort_77 => 77u32,
        abort_78 => 78u32,
        abort_79 => 79u32,

        abort_80 => 80u32,
        abort_81 => 81u32,
        abort_82 => 82u32,
        abort_83 => 83u32,
        abort_84 => 84u32,
        abort_85 => 85u32,
        abort_86 => 86u32,
        abort_87 => 87u32,
        abort_88 => 88u32,
        abort_89 => 89u32,

        abort_90 => 90u32,
        abort_91 => 91u32,
        abort_92 => 92u32,
        abort_93 => 93u32,
        abort_94 => 94u32,
        abort_95 => 95u32,
        abort_96 => 96u32,
        abort_97 => 97u32,
        abort_98 => 98u32,
        abort_99 => 99u32,

        abort_100 => 100u32,
        abort_101 => 101u32,
        abort_102 => 102u32,
        abort_103 => 103u32,
        abort_104 => 104u32,
        abort_105 => 105u32,
        abort_106 => 106u32,
        abort_107 => 107u32,
        abort_108 => 108u32,
        abort_109 => 109u32,

        abort_110 => 110u32,
        abort_111 => 111u32,
        abort_112 => 112u32,
        abort_113 => 113u32,
        abort_114 => 114u32,
        abort_115 => 115u32,
        abort_116 => 116u32,
        abort_117 => 117u32,
        abort_118 => 118u32,
        abort_119 => 119u32,

        abort_120 => 120u32,
        abort_121 => 121u32,
        abort_122 => 122u32,
        abort_123 => 123u32,
        abort_124 => 124u32,
        abort_125 => 125u32,
        abort_126 => 126u32,
        abort_127 => 127u32,
        abort_128 => 128u32,
        abort_129 => 129u32,

        abort_130 => 130u32,
        abort_131 => 131u32,
        abort_132 => 132u32,
        abort_133 => 133u32,
        abort_134 => 134u32,
        abort_135 => 135u32,
        abort_136 => 136u32,
        abort_137 => 137u32,
        abort_138 => 138u32,
        abort_139 => 139u32,

        abort_140 => 140u32,
        abort_141 => 141u32,
        abort_142 => 142u32,
        abort_143 => 143u32,
        abort_144 => 144u32,
        abort_145 => 145u32,
        abort_146 => 146u32,
        abort_147 => 147u32,
        abort_148 => 148u32,
        abort_149 => 149u32,

        abort_150 => 150u32,
        abort_151 => 151u32,
        abort_152 => 152u32,
        abort_153 => 153u32,
        abort_154 => 154u32,
        abort_155 => 155u32,
        abort_156 => 156u32,
        abort_157 => 157u32,
        abort_158 => 158u32,
        abort_159 => 159u32,

        abort_160 => 160u32,
        abort_161 => 161u32,
        abort_162 => 162u32,
        abort_163 => 163u32,
        abort_164 => 164u32,
        abort_165 => 165u32,
        abort_166 => 166u32,
        abort_167 => 167u32,
        abort_168 => 168u32,
        abort_169 => 169u32,

        abort_170 => 170u32,
        abort_171 => 171u32,
        abort_172 => 172u32,
        abort_173 => 173u32,
        abort_174 => 174u32,
        abort_175 => 175u32,
        abort_176 => 176u32,
        abort_177 => 177u32,
        abort_178 => 178u32,
        abort_179 => 179u32,

        abort_180 => 180u32,
        abort_181 => 181u32,
        abort_182 => 182u32,
        abort_183 => 183u32,
        abort_184 => 184u32,
        abort_185 => 185u32,
        abort_186 => 186u32,
        abort_187 => 187u32,
        abort_188 => 188u32,
        abort_189 => 189u32,

        abort_190 => 190u32,
        abort_191 => 191u32,
        abort_192 => 192u32,
        abort_193 => 193u32,
        abort_194 => 194u32,
        abort_195 => 195u32,
        abort_196 => 196u32,
        abort_197 => 197u32,
        abort_198 => 198u32,
        abort_199 => 199u32,

        abort_200 => 200u32,
        abort_201 => 201u32,
        abort_202 => 202u32,
        abort_203 => 203u32,
        abort_204 => 204u32,
        abort_205 => 205u32,
        abort_206 => 206u32,
        abort_207 => 207u32,
        abort_208 => 208u32,
        abort_209 => 209u32,

        abort_210 => 210u32,
        abort_211 => 211u32,
        abort_212 => 212u32,
        abort_213 => 213u32,
        abort_214 => 214u32,
        abort_215 => 215u32,
        abort_216 => 216u32,
        abort_217 => 217u32,
        abort_218 => 218u32,
        abort_219 => 219u32,

        abort_220 => 220u32,
        abort_221 => 221u32,
        abort_222 => 222u32,
        abort_223 => 223u32,
        abort_224 => 224u32,
        abort_225 => 225u32,
        abort_226 => 226u32,
        abort_227 => 227u32,
        abort_228 => 228u32,
        abort_229 => 229u32,

        abort_230 => 230u32,
        abort_231 => 231u32,
        abort_232 => 232u32,
        abort_233 => 233u32,
        abort_234 => 234u32,
        abort_235 => 235u32,
        abort_236 => 236u32,
        abort_237 => 237u32,
        abort_238 => 238u32,
        abort_239 => 239u32,

        abort_240 => 240u32,
        abort_241 => 241u32,
        abort_242 => 242u32,
        abort_243 => 243u32,
        abort_244 => 244u32,
        abort_245 => 245u32,
        abort_246 => 246u32,
        abort_247 => 247u32,
        abort_248 => 248u32,
        abort_249 => 249u32,

        abort_250 => 250u32,
        abort_251 => 251u32,
        abort_252 => 252u32,
        abort_253 => 253u32,
        abort_254 => 254u32,
        abort_255 => 255u32,
    }
}

/// States why the abort occured
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
pub enum AbortCode {
    /// Retry means you might want to attempt to start the
    /// transaction again.
    Retry = 2,

    /// Conflict implies another execute unit is modifying
    /// the data you're working with.
    Conflict = 4,

    /// Capacity means too much data is modified during
    /// the transaction.
    Capacity = 8,

    /// Debug implies a debugger interupted the transaction.
    Debug = 16,

    /// Nested states too many tsx transactions are being
    /// nested.
    Nested = 32,

    Code0 = crate::abort_codes::abort_0,
    Code1 = crate::abort_codes::abort_1,
    Code2 = crate::abort_codes::abort_2,
    Code3 = crate::abort_codes::abort_3,
    Code4 = crate::abort_codes::abort_4,
    Code5 = crate::abort_codes::abort_5,
    Code6 = crate::abort_codes::abort_6,
    Code7 = crate::abort_codes::abort_7,
    Code8 = crate::abort_codes::abort_8,
    Code9 = crate::abort_codes::abort_9,

    Code10 = crate::abort_codes::abort_10,
    Code11 = crate::abort_codes::abort_11,
    Code12 = crate::abort_codes::abort_12,
    Code13 = crate::abort_codes::abort_13,
    Code14 = crate::abort_codes::abort_14,
    Code15 = crate::abort_codes::abort_15,
    Code16 = crate::abort_codes::abort_16,
    Code17 = crate::abort_codes::abort_17,
    Code18 = crate::abort_codes::abort_18,
    Code19 = crate::abort_codes::abort_19,

    Code20 = crate::abort_codes::abort_20,
    Code21 = crate::abort_codes::abort_21,
    Code22 = crate::abort_codes::abort_22,
    Code23 = crate::abort_codes::abort_23,
    Code24 = crate::abort_codes::abort_24,
    Code25 = crate::abort_codes::abort_25,
    Code26 = crate::abort_codes::abort_26,
    Code27 = crate::abort_codes::abort_27,
    Code28 = crate::abort_codes::abort_28,
    Code29 = crate::abort_codes::abort_29,

    Code30 = crate::abort_codes::abort_30,
    Code31 = crate::abort_codes::abort_31,
    Code32 = crate::abort_codes::abort_32,
    Code33 = crate::abort_codes::abort_33,
    Code34 = crate::abort_codes::abort_34,
    Code35 = crate::abort_codes::abort_35,
    Code36 = crate::abort_codes::abort_36,
    Code37 = crate::abort_codes::abort_37,
    Code38 = crate::abort_codes::abort_38,
    Code39 = crate::abort_codes::abort_39,

    Code40 = crate::abort_codes::abort_40,
    Code41 = crate::abort_codes::abort_41,
    Code42 = crate::abort_codes::abort_42,
    Code43 = crate::abort_codes::abort_43,
    Code44 = crate::abort_codes::abort_44,
    Code45 = crate::abort_codes::abort_45,
    Code46 = crate::abort_codes::abort_46,
    Code47 = crate::abort_codes::abort_47,
    Code48 = crate::abort_codes::abort_48,
    Code49 = crate::abort_codes::abort_49,

    Code50 = crate::abort_codes::abort_50,
    Code51 = crate::abort_codes::abort_51,
    Code52 = crate::abort_codes::abort_52,
    Code53 = crate::abort_codes::abort_53,
    Code54 = crate::abort_codes::abort_54,
    Code55 = crate::abort_codes::abort_55,
    Code56 = crate::abort_codes::abort_56,
    Code57 = crate::abort_codes::abort_57,
    Code58 = crate::abort_codes::abort_58,
    Code59 = crate::abort_codes::abort_59,

    Code60 = crate::abort_codes::abort_60,
    Code61 = crate::abort_codes::abort_61,
    Code62 = crate::abort_codes::abort_62,
    Code63 = crate::abort_codes::abort_63,
    Code64 = crate::abort_codes::abort_64,
    Code65 = crate::abort_codes::abort_65,
    Code66 = crate::abort_codes::abort_66,
    Code67 = crate::abort_codes::abort_67,
    Code68 = crate::abort_codes::abort_68,
    Code69 = crate::abort_codes::abort_69,

    Code70 = crate::abort_codes::abort_70,
    Code71 = crate::abort_codes::abort_71,
    Code72 = crate::abort_codes::abort_72,
    Code73 = crate::abort_codes::abort_73,
    Code74 = crate::abort_codes::abort_74,
    Code75 = crate::abort_codes::abort_75,
    Code76 = crate::abort_codes::abort_76,
    Code77 = crate::abort_codes::abort_77,
    Code78 = crate::abort_codes::abort_78,
    Code79 = crate::abort_codes::abort_79,

    Code80 = crate::abort_codes::abort_80,
    Code81 = crate::abort_codes::abort_81,
    Code82 = crate::abort_codes::abort_82,
    Code83 = crate::abort_codes::abort_83,
    Code84 = crate::abort_codes::abort_84,
    Code85 = crate::abort_codes::abort_85,
    Code86 = crate::abort_codes::abort_86,
    Code87 = crate::abort_codes::abort_87,
    Code88 = crate::abort_codes::abort_88,
    Code89 = crate::abort_codes::abort_89,

    Code90 = crate::abort_codes::abort_90,
    Code91 = crate::abort_codes::abort_91,
    Code92 = crate::abort_codes::abort_92,
    Code93 = crate::abort_codes::abort_93,
    Code94 = crate::abort_codes::abort_94,
    Code95 = crate::abort_codes::abort_95,
    Code96 = crate::abort_codes::abort_96,
    Code97 = crate::abort_codes::abort_97,
    Code98 = crate::abort_codes::abort_98,
    Code99 = crate::abort_codes::abort_99,

    Code100 = crate::abort_codes::abort_100,
    Code101 = crate::abort_codes::abort_101,
    Code102 = crate::abort_codes::abort_102,
    Code103 = crate::abort_codes::abort_103,
    Code104 = crate::abort_codes::abort_104,
    Code105 = crate::abort_codes::abort_105,
    Code106 = crate::abort_codes::abort_106,
    Code107 = crate::abort_codes::abort_107,
    Code108 = crate::abort_codes::abort_108,
    Code109 = crate::abort_codes::abort_109,

    Code110 = crate::abort_codes::abort_110,
    Code111 = crate::abort_codes::abort_111,
    Code112 = crate::abort_codes::abort_112,
    Code113 = crate::abort_codes::abort_113,
    Code114 = crate::abort_codes::abort_114,
    Code115 = crate::abort_codes::abort_115,
    Code116 = crate::abort_codes::abort_116,
    Code117 = crate::abort_codes::abort_117,
    Code118 = crate::abort_codes::abort_118,
    Code119 = crate::abort_codes::abort_119,

    Code120 = crate::abort_codes::abort_120,
    Code121 = crate::abort_codes::abort_121,
    Code122 = crate::abort_codes::abort_122,
    Code123 = crate::abort_codes::abort_123,
    Code124 = crate::abort_codes::abort_124,
    Code125 = crate::abort_codes::abort_125,
    Code126 = crate::abort_codes::abort_126,
    Code127 = crate::abort_codes::abort_127,
    Code128 = crate::abort_codes::abort_128,
    Code129 = crate::abort_codes::abort_129,

    Code130 = crate::abort_codes::abort_130,
    Code131 = crate::abort_codes::abort_131,
    Code132 = crate::abort_codes::abort_132,
    Code133 = crate::abort_codes::abort_133,
    Code134 = crate::abort_codes::abort_134,
    Code135 = crate::abort_codes::abort_135,
    Code136 = crate::abort_codes::abort_136,
    Code137 = crate::abort_codes::abort_137,
    Code138 = crate::abort_codes::abort_138,
    Code139 = crate::abort_codes::abort_139,

    Code140 = crate::abort_codes::abort_140,
    Code141 = crate::abort_codes::abort_141,
    Code142 = crate::abort_codes::abort_142,
    Code143 = crate::abort_codes::abort_143,
    Code144 = crate::abort_codes::abort_144,
    Code145 = crate::abort_codes::abort_145,
    Code146 = crate::abort_codes::abort_146,
    Code147 = crate::abort_codes::abort_147,
    Code148 = crate::abort_codes::abort_148,
    Code149 = crate::abort_codes::abort_149,

    Code150 = crate::abort_codes::abort_150,
    Code151 = crate::abort_codes::abort_151,
    Code152 = crate::abort_codes::abort_152,
    Code153 = crate::abort_codes::abort_153,
    Code154 = crate::abort_codes::abort_154,
    Code155 = crate::abort_codes::abort_155,
    Code156 = crate::abort_codes::abort_156,
    Code157 = crate::abort_codes::abort_157,
    Code158 = crate::abort_codes::abort_158,
    Code159 = crate::abort_codes::abort_159,

    Code160 = crate::abort_codes::abort_160,
    Code161 = crate::abort_codes::abort_161,
    Code162 = crate::abort_codes::abort_162,
    Code163 = crate::abort_codes::abort_163,
    Code164 = crate::abort_codes::abort_164,
    Code165 = crate::abort_codes::abort_165,
    Code166 = crate::abort_codes::abort_166,
    Code167 = crate::abort_codes::abort_167,
    Code168 = crate::abort_codes::abort_168,
    Code169 = crate::abort_codes::abort_169,

    Code170 = crate::abort_codes::abort_170,
    Code171 = crate::abort_codes::abort_171,
    Code172 = crate::abort_codes::abort_172,
    Code173 = crate::abort_codes::abort_173,
    Code174 = crate::abort_codes::abort_174,
    Code175 = crate::abort_codes::abort_175,
    Code176 = crate::abort_codes::abort_176,
    Code177 = crate::abort_codes::abort_177,
    Code178 = crate::abort_codes::abort_178,
    Code179 = crate::abort_codes::abort_179,

    Code180 = crate::abort_codes::abort_180,
    Code181 = crate::abort_codes::abort_181,
    Code182 = crate::abort_codes::abort_182,
    Code183 = crate::abort_codes::abort_183,
    Code184 = crate::abort_codes::abort_184,
    Code185 = crate::abort_codes::abort_185,
    Code186 = crate::abort_codes::abort_186,
    Code187 = crate::abort_codes::abort_187,
    Code188 = crate::abort_codes::abort_188,
    Code189 = crate::abort_codes::abort_189,

    Code190 = crate::abort_codes::abort_190,
    Code191 = crate::abort_codes::abort_191,
    Code192 = crate::abort_codes::abort_192,
    Code193 = crate::abort_codes::abort_193,
    Code194 = crate::abort_codes::abort_194,
    Code195 = crate::abort_codes::abort_195,
    Code196 = crate::abort_codes::abort_196,
    Code197 = crate::abort_codes::abort_197,
    Code198 = crate::abort_codes::abort_198,
    Code199 = crate::abort_codes::abort_199,

    Code200 = crate::abort_codes::abort_200,
    Code201 = crate::abort_codes::abort_201,
    Code202 = crate::abort_codes::abort_202,
    Code203 = crate::abort_codes::abort_203,
    Code204 = crate::abort_codes::abort_204,
    Code205 = crate::abort_codes::abort_205,
    Code206 = crate::abort_codes::abort_206,
    Code207 = crate::abort_codes::abort_207,
    Code208 = crate::abort_codes::abort_208,
    Code209 = crate::abort_codes::abort_209,

    Code210 = crate::abort_codes::abort_210,
    Code211 = crate::abort_codes::abort_211,
    Code212 = crate::abort_codes::abort_212,
    Code213 = crate::abort_codes::abort_213,
    Code214 = crate::abort_codes::abort_214,
    Code215 = crate::abort_codes::abort_215,
    Code216 = crate::abort_codes::abort_216,
    Code217 = crate::abort_codes::abort_217,
    Code218 = crate::abort_codes::abort_218,
    Code219 = crate::abort_codes::abort_219,

    Code220 = crate::abort_codes::abort_220,
    Code221 = crate::abort_codes::abort_221,
    Code222 = crate::abort_codes::abort_222,
    Code223 = crate::abort_codes::abort_223,
    Code224 = crate::abort_codes::abort_224,
    Code225 = crate::abort_codes::abort_225,
    Code226 = crate::abort_codes::abort_226,
    Code227 = crate::abort_codes::abort_227,
    Code228 = crate::abort_codes::abort_228,
    Code229 = crate::abort_codes::abort_229,

    Code230 = crate::abort_codes::abort_230,
    Code231 = crate::abort_codes::abort_231,
    Code232 = crate::abort_codes::abort_232,
    Code233 = crate::abort_codes::abort_233,
    Code234 = crate::abort_codes::abort_234,
    Code235 = crate::abort_codes::abort_235,
    Code236 = crate::abort_codes::abort_236,
    Code237 = crate::abort_codes::abort_237,
    Code238 = crate::abort_codes::abort_238,
    Code239 = crate::abort_codes::abort_239,

    Code240 = crate::abort_codes::abort_240,
    Code241 = crate::abort_codes::abort_241,
    Code242 = crate::abort_codes::abort_242,
    Code243 = crate::abort_codes::abort_243,
    Code244 = crate::abort_codes::abort_244,
    Code245 = crate::abort_codes::abort_245,
    Code246 = crate::abort_codes::abort_246,
    Code247 = crate::abort_codes::abort_247,
    Code248 = crate::abort_codes::abort_248,
    Code249 = crate::abort_codes::abort_249,

    Code250 = crate::abort_codes::abort_250,
    Code251 = crate::abort_codes::abort_251,
    Code252 = crate::abort_codes::abort_252,
    Code253 = crate::abort_codes::abort_253,
    Code254 = crate::abort_codes::abort_254,
    Code255 = crate::abort_codes::abort_255,
}
impl AbortCode {
    /// converts the
    #[inline]
    pub fn into_code(&self) -> Option<u8> {
        match self {
            &Self::Retry | &Self::Conflict | &Self::Capacity | &Self::Debug | &Self::Nested => None,
            arg => {
                let value: u32 = *arg as u32;
                Some(((value - 1) >> 24) as u8)
            }
        }
    }
}

/// handles the messiness of converting a abort code
#[allow(dead_code)]
#[inline(always)]
fn into_abort(x: u32) -> Result<(), AbortCode> {
    match x {
        2 => Err(AbortCode::Retry),
        4 => Err(AbortCode::Conflict),
        8 => Err(AbortCode::Capacity),
        16 => Err(AbortCode::Debug),
        32 => Err(AbortCode::Nested),
        crate::abort_codes::abort_0 => Err(AbortCode::Code0),
        crate::abort_codes::abort_1 => Err(AbortCode::Code1),
        crate::abort_codes::abort_2 => Err(AbortCode::Code2),
        crate::abort_codes::abort_3 => Err(AbortCode::Code3),
        crate::abort_codes::abort_4 => Err(AbortCode::Code4),
        crate::abort_codes::abort_5 => Err(AbortCode::Code5),
        crate::abort_codes::abort_6 => Err(AbortCode::Code6),
        crate::abort_codes::abort_7 => Err(AbortCode::Code7),
        crate::abort_codes::abort_8 => Err(AbortCode::Code8),
        crate::abort_codes::abort_9 => Err(AbortCode::Code9),
        crate::abort_codes::abort_10 => Err(AbortCode::Code10),
        crate::abort_codes::abort_11 => Err(AbortCode::Code11),
        crate::abort_codes::abort_12 => Err(AbortCode::Code12),
        crate::abort_codes::abort_13 => Err(AbortCode::Code13),
        crate::abort_codes::abort_14 => Err(AbortCode::Code14),
        crate::abort_codes::abort_15 => Err(AbortCode::Code15),
        crate::abort_codes::abort_16 => Err(AbortCode::Code16),
        crate::abort_codes::abort_17 => Err(AbortCode::Code17),
        crate::abort_codes::abort_18 => Err(AbortCode::Code18),
        crate::abort_codes::abort_19 => Err(AbortCode::Code19),
        crate::abort_codes::abort_20 => Err(AbortCode::Code20),
        crate::abort_codes::abort_21 => Err(AbortCode::Code21),
        crate::abort_codes::abort_22 => Err(AbortCode::Code22),
        crate::abort_codes::abort_23 => Err(AbortCode::Code23),
        crate::abort_codes::abort_24 => Err(AbortCode::Code24),
        crate::abort_codes::abort_25 => Err(AbortCode::Code25),
        crate::abort_codes::abort_26 => Err(AbortCode::Code26),
        crate::abort_codes::abort_27 => Err(AbortCode::Code27),
        crate::abort_codes::abort_28 => Err(AbortCode::Code28),
        crate::abort_codes::abort_29 => Err(AbortCode::Code29),

        crate::abort_codes::abort_30 => Err(AbortCode::Code30),
        crate::abort_codes::abort_31 => Err(AbortCode::Code31),
        crate::abort_codes::abort_32 => Err(AbortCode::Code32),
        crate::abort_codes::abort_33 => Err(AbortCode::Code33),
        crate::abort_codes::abort_34 => Err(AbortCode::Code34),
        crate::abort_codes::abort_35 => Err(AbortCode::Code35),
        crate::abort_codes::abort_36 => Err(AbortCode::Code36),
        crate::abort_codes::abort_37 => Err(AbortCode::Code37),
        crate::abort_codes::abort_38 => Err(AbortCode::Code38),
        crate::abort_codes::abort_39 => Err(AbortCode::Code39),

        crate::abort_codes::abort_40 => Err(AbortCode::Code40),
        crate::abort_codes::abort_41 => Err(AbortCode::Code41),
        crate::abort_codes::abort_42 => Err(AbortCode::Code42),
        crate::abort_codes::abort_43 => Err(AbortCode::Code43),
        crate::abort_codes::abort_44 => Err(AbortCode::Code44),
        crate::abort_codes::abort_45 => Err(AbortCode::Code45),
        crate::abort_codes::abort_46 => Err(AbortCode::Code46),
        crate::abort_codes::abort_47 => Err(AbortCode::Code47),
        crate::abort_codes::abort_48 => Err(AbortCode::Code48),
        crate::abort_codes::abort_49 => Err(AbortCode::Code49),

        crate::abort_codes::abort_50 => Err(AbortCode::Code50),
        crate::abort_codes::abort_51 => Err(AbortCode::Code51),
        crate::abort_codes::abort_52 => Err(AbortCode::Code52),
        crate::abort_codes::abort_53 => Err(AbortCode::Code53),
        crate::abort_codes::abort_54 => Err(AbortCode::Code54),
        crate::abort_codes::abort_55 => Err(AbortCode::Code55),
        crate::abort_codes::abort_56 => Err(AbortCode::Code56),
        crate::abort_codes::abort_57 => Err(AbortCode::Code57),
        crate::abort_codes::abort_58 => Err(AbortCode::Code58),
        crate::abort_codes::abort_59 => Err(AbortCode::Code59),

        crate::abort_codes::abort_60 => Err(AbortCode::Code60),
        crate::abort_codes::abort_61 => Err(AbortCode::Code61),
        crate::abort_codes::abort_62 => Err(AbortCode::Code62),
        crate::abort_codes::abort_63 => Err(AbortCode::Code63),
        crate::abort_codes::abort_64 => Err(AbortCode::Code64),
        crate::abort_codes::abort_65 => Err(AbortCode::Code65),
        crate::abort_codes::abort_66 => Err(AbortCode::Code66),
        crate::abort_codes::abort_67 => Err(AbortCode::Code67),
        crate::abort_codes::abort_68 => Err(AbortCode::Code68),
        crate::abort_codes::abort_69 => Err(AbortCode::Code69),

        crate::abort_codes::abort_70 => Err(AbortCode::Code70),
        crate::abort_codes::abort_71 => Err(AbortCode::Code71),
        crate::abort_codes::abort_72 => Err(AbortCode::Code72),
        crate::abort_codes::abort_73 => Err(AbortCode::Code73),
        crate::abort_codes::abort_74 => Err(AbortCode::Code74),
        crate::abort_codes::abort_75 => Err(AbortCode::Code75),
        crate::abort_codes::abort_76 => Err(AbortCode::Code76),
        crate::abort_codes::abort_77 => Err(AbortCode::Code77),
        crate::abort_codes::abort_78 => Err(AbortCode::Code78),
        crate::abort_codes::abort_79 => Err(AbortCode::Code79),

        crate::abort_codes::abort_80 => Err(AbortCode::Code80),
        crate::abort_codes::abort_81 => Err(AbortCode::Code81),
        crate::abort_codes::abort_82 => Err(AbortCode::Code82),
        crate::abort_codes::abort_83 => Err(AbortCode::Code83),
        crate::abort_codes::abort_84 => Err(AbortCode::Code84),
        crate::abort_codes::abort_85 => Err(AbortCode::Code85),
        crate::abort_codes::abort_86 => Err(AbortCode::Code86),
        crate::abort_codes::abort_87 => Err(AbortCode::Code87),
        crate::abort_codes::abort_88 => Err(AbortCode::Code88),
        crate::abort_codes::abort_89 => Err(AbortCode::Code89),

        crate::abort_codes::abort_90 => Err(AbortCode::Code90),
        crate::abort_codes::abort_91 => Err(AbortCode::Code91),
        crate::abort_codes::abort_92 => Err(AbortCode::Code92),
        crate::abort_codes::abort_93 => Err(AbortCode::Code93),
        crate::abort_codes::abort_94 => Err(AbortCode::Code94),
        crate::abort_codes::abort_95 => Err(AbortCode::Code95),
        crate::abort_codes::abort_96 => Err(AbortCode::Code96),
        crate::abort_codes::abort_97 => Err(AbortCode::Code97),
        crate::abort_codes::abort_98 => Err(AbortCode::Code98),
        crate::abort_codes::abort_99 => Err(AbortCode::Code99),

        crate::abort_codes::abort_100 => Err(AbortCode::Code100),
        crate::abort_codes::abort_101 => Err(AbortCode::Code101),
        crate::abort_codes::abort_102 => Err(AbortCode::Code102),
        crate::abort_codes::abort_103 => Err(AbortCode::Code103),
        crate::abort_codes::abort_104 => Err(AbortCode::Code104),
        crate::abort_codes::abort_105 => Err(AbortCode::Code105),
        crate::abort_codes::abort_106 => Err(AbortCode::Code106),
        crate::abort_codes::abort_107 => Err(AbortCode::Code107),
        crate::abort_codes::abort_108 => Err(AbortCode::Code108),
        crate::abort_codes::abort_109 => Err(AbortCode::Code109),

        crate::abort_codes::abort_110 => Err(AbortCode::Code110),
        crate::abort_codes::abort_111 => Err(AbortCode::Code111),
        crate::abort_codes::abort_112 => Err(AbortCode::Code112),
        crate::abort_codes::abort_113 => Err(AbortCode::Code113),
        crate::abort_codes::abort_114 => Err(AbortCode::Code114),
        crate::abort_codes::abort_115 => Err(AbortCode::Code115),
        crate::abort_codes::abort_116 => Err(AbortCode::Code116),
        crate::abort_codes::abort_117 => Err(AbortCode::Code117),
        crate::abort_codes::abort_118 => Err(AbortCode::Code118),
        crate::abort_codes::abort_119 => Err(AbortCode::Code119),

        crate::abort_codes::abort_120 => Err(AbortCode::Code120),
        crate::abort_codes::abort_121 => Err(AbortCode::Code121),
        crate::abort_codes::abort_122 => Err(AbortCode::Code122),
        crate::abort_codes::abort_123 => Err(AbortCode::Code123),
        crate::abort_codes::abort_124 => Err(AbortCode::Code124),
        crate::abort_codes::abort_125 => Err(AbortCode::Code125),
        crate::abort_codes::abort_126 => Err(AbortCode::Code126),
        crate::abort_codes::abort_127 => Err(AbortCode::Code127),
        crate::abort_codes::abort_128 => Err(AbortCode::Code128),
        crate::abort_codes::abort_129 => Err(AbortCode::Code129),

        crate::abort_codes::abort_130 => Err(AbortCode::Code130),
        crate::abort_codes::abort_131 => Err(AbortCode::Code131),
        crate::abort_codes::abort_132 => Err(AbortCode::Code132),
        crate::abort_codes::abort_133 => Err(AbortCode::Code133),
        crate::abort_codes::abort_134 => Err(AbortCode::Code134),
        crate::abort_codes::abort_135 => Err(AbortCode::Code135),
        crate::abort_codes::abort_136 => Err(AbortCode::Code136),
        crate::abort_codes::abort_137 => Err(AbortCode::Code137),
        crate::abort_codes::abort_138 => Err(AbortCode::Code138),
        crate::abort_codes::abort_139 => Err(AbortCode::Code139),

        crate::abort_codes::abort_140 => Err(AbortCode::Code140),
        crate::abort_codes::abort_141 => Err(AbortCode::Code141),
        crate::abort_codes::abort_142 => Err(AbortCode::Code142),
        crate::abort_codes::abort_143 => Err(AbortCode::Code143),
        crate::abort_codes::abort_144 => Err(AbortCode::Code144),
        crate::abort_codes::abort_145 => Err(AbortCode::Code145),
        crate::abort_codes::abort_146 => Err(AbortCode::Code146),
        crate::abort_codes::abort_147 => Err(AbortCode::Code147),
        crate::abort_codes::abort_148 => Err(AbortCode::Code148),
        crate::abort_codes::abort_149 => Err(AbortCode::Code149),

        crate::abort_codes::abort_150 => Err(AbortCode::Code150),
        crate::abort_codes::abort_151 => Err(AbortCode::Code151),
        crate::abort_codes::abort_152 => Err(AbortCode::Code152),
        crate::abort_codes::abort_153 => Err(AbortCode::Code153),
        crate::abort_codes::abort_154 => Err(AbortCode::Code154),
        crate::abort_codes::abort_155 => Err(AbortCode::Code155),
        crate::abort_codes::abort_156 => Err(AbortCode::Code156),
        crate::abort_codes::abort_157 => Err(AbortCode::Code157),
        crate::abort_codes::abort_158 => Err(AbortCode::Code158),
        crate::abort_codes::abort_159 => Err(AbortCode::Code159),

        crate::abort_codes::abort_160 => Err(AbortCode::Code160),
        crate::abort_codes::abort_161 => Err(AbortCode::Code161),
        crate::abort_codes::abort_162 => Err(AbortCode::Code162),
        crate::abort_codes::abort_163 => Err(AbortCode::Code163),
        crate::abort_codes::abort_164 => Err(AbortCode::Code164),
        crate::abort_codes::abort_165 => Err(AbortCode::Code165),
        crate::abort_codes::abort_166 => Err(AbortCode::Code166),
        crate::abort_codes::abort_167 => Err(AbortCode::Code167),
        crate::abort_codes::abort_168 => Err(AbortCode::Code168),
        crate::abort_codes::abort_169 => Err(AbortCode::Code169),

        crate::abort_codes::abort_170 => Err(AbortCode::Code170),
        crate::abort_codes::abort_171 => Err(AbortCode::Code171),
        crate::abort_codes::abort_172 => Err(AbortCode::Code172),
        crate::abort_codes::abort_173 => Err(AbortCode::Code173),
        crate::abort_codes::abort_174 => Err(AbortCode::Code174),
        crate::abort_codes::abort_175 => Err(AbortCode::Code175),
        crate::abort_codes::abort_176 => Err(AbortCode::Code176),
        crate::abort_codes::abort_177 => Err(AbortCode::Code177),
        crate::abort_codes::abort_178 => Err(AbortCode::Code178),
        crate::abort_codes::abort_179 => Err(AbortCode::Code179),

        crate::abort_codes::abort_180 => Err(AbortCode::Code180),
        crate::abort_codes::abort_181 => Err(AbortCode::Code181),
        crate::abort_codes::abort_182 => Err(AbortCode::Code182),
        crate::abort_codes::abort_183 => Err(AbortCode::Code183),
        crate::abort_codes::abort_184 => Err(AbortCode::Code184),
        crate::abort_codes::abort_185 => Err(AbortCode::Code185),
        crate::abort_codes::abort_186 => Err(AbortCode::Code186),
        crate::abort_codes::abort_187 => Err(AbortCode::Code187),
        crate::abort_codes::abort_188 => Err(AbortCode::Code188),
        crate::abort_codes::abort_189 => Err(AbortCode::Code189),

        crate::abort_codes::abort_190 => Err(AbortCode::Code190),
        crate::abort_codes::abort_191 => Err(AbortCode::Code191),
        crate::abort_codes::abort_192 => Err(AbortCode::Code192),
        crate::abort_codes::abort_193 => Err(AbortCode::Code193),
        crate::abort_codes::abort_194 => Err(AbortCode::Code194),
        crate::abort_codes::abort_195 => Err(AbortCode::Code195),
        crate::abort_codes::abort_196 => Err(AbortCode::Code196),
        crate::abort_codes::abort_197 => Err(AbortCode::Code197),
        crate::abort_codes::abort_198 => Err(AbortCode::Code198),
        crate::abort_codes::abort_199 => Err(AbortCode::Code199),

        crate::abort_codes::abort_200 => Err(AbortCode::Code200),
        crate::abort_codes::abort_201 => Err(AbortCode::Code201),
        crate::abort_codes::abort_202 => Err(AbortCode::Code202),
        crate::abort_codes::abort_203 => Err(AbortCode::Code203),
        crate::abort_codes::abort_204 => Err(AbortCode::Code204),
        crate::abort_codes::abort_205 => Err(AbortCode::Code205),
        crate::abort_codes::abort_206 => Err(AbortCode::Code206),
        crate::abort_codes::abort_207 => Err(AbortCode::Code207),
        crate::abort_codes::abort_208 => Err(AbortCode::Code208),
        crate::abort_codes::abort_209 => Err(AbortCode::Code209),

        crate::abort_codes::abort_210 => Err(AbortCode::Code210),
        crate::abort_codes::abort_211 => Err(AbortCode::Code211),
        crate::abort_codes::abort_212 => Err(AbortCode::Code212),
        crate::abort_codes::abort_213 => Err(AbortCode::Code213),
        crate::abort_codes::abort_214 => Err(AbortCode::Code214),
        crate::abort_codes::abort_215 => Err(AbortCode::Code215),
        crate::abort_codes::abort_216 => Err(AbortCode::Code216),
        crate::abort_codes::abort_217 => Err(AbortCode::Code217),
        crate::abort_codes::abort_218 => Err(AbortCode::Code218),
        crate::abort_codes::abort_219 => Err(AbortCode::Code219),

        crate::abort_codes::abort_220 => Err(AbortCode::Code220),
        crate::abort_codes::abort_221 => Err(AbortCode::Code221),
        crate::abort_codes::abort_222 => Err(AbortCode::Code222),
        crate::abort_codes::abort_223 => Err(AbortCode::Code223),
        crate::abort_codes::abort_224 => Err(AbortCode::Code224),
        crate::abort_codes::abort_225 => Err(AbortCode::Code225),
        crate::abort_codes::abort_226 => Err(AbortCode::Code226),
        crate::abort_codes::abort_227 => Err(AbortCode::Code227),
        crate::abort_codes::abort_228 => Err(AbortCode::Code228),
        crate::abort_codes::abort_229 => Err(AbortCode::Code229),

        crate::abort_codes::abort_230 => Err(AbortCode::Code230),
        crate::abort_codes::abort_231 => Err(AbortCode::Code231),
        crate::abort_codes::abort_232 => Err(AbortCode::Code232),
        crate::abort_codes::abort_233 => Err(AbortCode::Code233),
        crate::abort_codes::abort_234 => Err(AbortCode::Code234),
        crate::abort_codes::abort_235 => Err(AbortCode::Code235),
        crate::abort_codes::abort_236 => Err(AbortCode::Code236),
        crate::abort_codes::abort_237 => Err(AbortCode::Code237),
        crate::abort_codes::abort_238 => Err(AbortCode::Code238),
        crate::abort_codes::abort_239 => Err(AbortCode::Code239),

        crate::abort_codes::abort_240 => Err(AbortCode::Code240),
        crate::abort_codes::abort_241 => Err(AbortCode::Code241),
        crate::abort_codes::abort_242 => Err(AbortCode::Code242),
        crate::abort_codes::abort_243 => Err(AbortCode::Code243),
        crate::abort_codes::abort_244 => Err(AbortCode::Code244),
        crate::abort_codes::abort_245 => Err(AbortCode::Code245),
        crate::abort_codes::abort_246 => Err(AbortCode::Code246),
        crate::abort_codes::abort_247 => Err(AbortCode::Code247),
        crate::abort_codes::abort_248 => Err(AbortCode::Code248),
        crate::abort_codes::abort_249 => Err(AbortCode::Code249),

        crate::abort_codes::abort_250 => Err(AbortCode::Code250),
        crate::abort_codes::abort_251 => Err(AbortCode::Code251),
        crate::abort_codes::abort_252 => Err(AbortCode::Code252),
        crate::abort_codes::abort_253 => Err(AbortCode::Code253),
        crate::abort_codes::abort_254 => Err(AbortCode::Code254),
        crate::abort_codes::abort_255 => Err(AbortCode::Code255),
        _ => {
            #[cfg(feature = "std")]
            unsafe {
                std::hint::unreachable_unchecked()
            }

            #[cfg(not(feature = "std"))]
            unsafe {
                core::hint::unreachable_unchecked()
            }
        }
    }
}

/*
/// Execute a transaction
///
/// This accepts data and a lambda function. It will return if the operations
/// succeeded or not, and _how_ it failed if it did.
#[inline(always)]
pub fn transaction<R: Sync, F: Fn(&mut R)>(lambda: &F, data: &mut R) -> Result<(), Abort> {
    //bit masks will be reduced to to constants at compile time
    let explicit: i32 = 1 << 0;
    let retry: i32 = 1 << 1;
    let conflict: i32 = 1 << 2;
    let capacity: i32 = 1 << 3;
    let debug: i32 = 1 << 4;
    let nested: i32 = 1 << 5;
    let mut out: Result<(), Abort> = Ok(());
    match unsafe { crate::tsx::_xbegin() } {
        -1 => {
            lambda(data);
            crate::tsx::_xend();
        }
        x if (x & retry) > 0 => out = Err(Abort::Retry),
        x if (x & conflict) > 0 => out = Err(Abort::Conflict),
        x if (x & capacity) > 0 => out = Err(Abort::Capacity),
        x if (x & debug) > 0 => out = Err(Abort::Debug),
        x if (x & nested) > 0 => out = Err(Abort::Nested),
        x if (x & explicit) > 0 => {
            out = Err(Abort::Code(((x >> 24) & 0xFF) as i8));
        }
        _ => out = Err(Abort::Undefined),
    };
    out
}
*/

/// Raw extension bindings
///
/// If a developer would rather roll their own
/// implementation without all the branching
/// and masking check this library does
/// internally.
///
/// If you would like to use these please see:
///
/// [Clang Reference](http://clang.llvm.org/doxygen/rtmintrin_8h.html)
///
/// [GCC Reference](https://gcc.gnu.org/onlinedocs/gcc-4.8.2/gcc/X86-transactional-memory-intrinsics.html)
///
/// [Intel Intrinsic Reference](https://software.intel.com/sites/landingpage/IntrinsicsGuide/#othertechs=RTM)
///
/// [Dr Dobb's Crash Course](http://www.drdobbs.com/parallel/transactional-synchronization-in-haswell/232600598)
///
pub mod tsx {

    #[cfg(not(feature = "std"))]
    pub use core::arch::x86_64::{_xabort, _xbegin, _xend, _xtest};

    #[cfg(feature = "std")]
    pub use std::arch::x86_64::{_xabort, _xbegin, _xend, _xtest};
}
