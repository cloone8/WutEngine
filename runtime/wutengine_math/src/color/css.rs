//! The CSS named colours, from CSS Color 4 (§6.1, named colors), as linear [`Color`]s.
//!
//! The names and values are the specification's; each constant is its sRGB value decoded to linear, precomputed
//! because the sRGB curve can't run in a `const`. Some colours have two names, as in CSS (`AQUA` and `CYAN`,
//! `FUCHSIA` and `MAGENTA`, the `GRAY` and `GREY` spellings). `TRANSPARENT` is on [`Color`] itself.

use super::Color;

/// CSS `aliceblue`, `#f0f8ff`
pub const ALICEBLUE: Color = Color::linear(0.871_367_1, 0.938_685_7, 1.0);

/// CSS `antiquewhite`, `#faebd7`
pub const ANTIQUEWHITE: Color = Color::linear(0.955_973_3, 0.830_769_9, 0.679_542_5);

/// CSS `aqua`, `#00ffff`
pub const AQUA: Color = Color::linear(0.0, 1.0, 1.0);

/// CSS `aquamarine`, `#7fffd4`
pub const AQUAMARINE: Color = Color::linear(0.212_230_76, 1.0, 0.658_374_85);

/// CSS `azure`, `#f0ffff`
pub const AZURE: Color = Color::linear(0.871_367_1, 1.0, 1.0);

/// CSS `beige`, `#f5f5dc`
pub const BEIGE: Color = Color::linear(0.913_098_63, 0.913_098_63, 0.715_693_5);

/// CSS `bisque`, `#ffe4c4`
pub const BISQUE: Color = Color::linear(1.0, 0.775_822_2, 0.552_011_43);

/// CSS `black`, `#000000`
pub const BLACK: Color = Color::linear(0.0, 0.0, 0.0);

/// CSS `blanchedalmond`, `#ffebcd`
pub const BLANCHEDALMOND: Color = Color::linear(1.0, 0.830_769_9, 0.610_495_57);

/// CSS `blue`, `#0000ff`
pub const BLUE: Color = Color::linear(0.0, 0.0, 1.0);

/// CSS `blueviolet`, `#8a2be2`
pub const BLUEVIOLET: Color = Color::linear(0.254_152_1, 0.024_157_632, 0.760_524_5);

/// CSS `brown`, `#a52a2a`
pub const BROWN: Color = Color::linear(0.376_262_13, 0.023_153_367, 0.023_153_367);

/// CSS `burlywood`, `#deb887`
pub const BURLYWOOD: Color = Color::linear(0.730_460_76, 0.479_320_17, 0.242_281_12);

/// CSS `cadetblue`, `#5f9ea0`
pub const CADETBLUE: Color = Color::linear(0.114_435_375, 0.341_914_42, 0.351_532_6);

/// CSS `chartreuse`, `#7fff00`
pub const CHARTREUSE: Color = Color::linear(0.212_230_76, 1.0, 0.0);

/// CSS `chocolate`, `#d2691e`
pub const CHOCOLATE: Color = Color::linear(0.644_479_7, 0.141_263_29, 0.012_983_032_5);

/// CSS `coral`, `#ff7f50`
pub const CORAL: Color = Color::linear(1.0, 0.212_230_76, 0.080_219_82);

/// CSS `cornflowerblue`, `#6495ed`
pub const CORNFLOWERBLUE: Color = Color::linear(0.127_437_68, 0.300_543_79, 0.846_873_2);

/// CSS `cornsilk`, `#fff8dc`
pub const CORNSILK: Color = Color::linear(1.0, 0.938_685_7, 0.715_693_5);

/// CSS `crimson`, `#dc143c`
pub const CRIMSON: Color = Color::linear(0.715_693_5, 0.006_995_41, 0.045_186_203);

/// CSS `cyan`, `#00ffff`
pub const CYAN: Color = Color::linear(0.0, 1.0, 1.0);

/// CSS `darkblue`, `#00008b`
pub const DARKBLUE: Color = Color::linear(0.0, 0.0, 0.258_182_85);

/// CSS `darkcyan`, `#008b8b`
pub const DARKCYAN: Color = Color::linear(0.0, 0.258_182_85, 0.258_182_85);

/// CSS `darkgoldenrod`, `#b8860b`
pub const DARKGOLDENROD: Color = Color::linear(0.479_320_17, 0.238_397_57, 0.003_346_535_8);

/// CSS `darkgray`, `#a9a9a9`
pub const DARKGRAY: Color = Color::linear(0.396_755_22, 0.396_755_22, 0.396_755_22);

/// CSS `darkgreen`, `#006400`
pub const DARKGREEN: Color = Color::linear(0.0, 0.127_437_68, 0.0);

/// CSS `darkgrey`, `#a9a9a9`
pub const DARKGREY: Color = Color::linear(0.396_755_22, 0.396_755_22, 0.396_755_22);

/// CSS `darkkhaki`, `#bdb76b`
pub const DARKKHAKI: Color = Color::linear(0.508_881_33, 0.473_531_48, 0.147_027_27);

/// CSS `darkmagenta`, `#8b008b`
pub const DARKMAGENTA: Color = Color::linear(0.258_182_85, 0.0, 0.258_182_85);

/// CSS `darkolivegreen`, `#556b2f`
pub const DARKOLIVEGREEN: Color = Color::linear(0.090_841_71, 0.147_027_27, 0.028_426_04);

/// CSS `darkorange`, `#ff8c00`
pub const DARKORANGE: Color = Color::linear(1.0, 0.262_250_66, 0.0);

/// CSS `darkorchid`, `#9932cc`
pub const DARKORCHID: Color = Color::linear(0.318_546_77, 0.031_896_032, 0.603_827_36);

/// CSS `darkred`, `#8b0000`
pub const DARKRED: Color = Color::linear(0.258_182_85, 0.0, 0.0);

/// CSS `darksalmon`, `#e9967a`
pub const DARKSALMON: Color = Color::linear(0.814_846_6, 0.304_987_3, 0.194_617_84);

/// CSS `darkseagreen`, `#8fbc8f`
pub const DARKSEAGREEN: Color = Color::linear(0.274_677_3, 0.502_886_5, 0.274_677_3);

/// CSS `darkslateblue`, `#483d8b`
pub const DARKSLATEBLUE: Color = Color::linear(0.064_803_265, 0.046_665_087, 0.258_182_85);

/// CSS `darkslategray`, `#2f4f4f`
pub const DARKSLATEGRAY: Color = Color::linear(0.028_426_04, 0.078_187_42, 0.078_187_42);

/// CSS `darkslategrey`, `#2f4f4f`
pub const DARKSLATEGREY: Color = Color::linear(0.028_426_04, 0.078_187_42, 0.078_187_42);

/// CSS `darkturquoise`, `#00ced1`
pub const DARKTURQUOISE: Color = Color::linear(0.0, 0.617_206_6, 0.637_596_85);

/// CSS `darkviolet`, `#9400d3`
pub const DARKVIOLET: Color = Color::linear(0.296_138_26, 0.0, 0.651_405_63);

/// CSS `deeppink`, `#ff1493`
pub const DEEPPINK: Color = Color::linear(1.0, 0.006_995_41, 0.291_770_64);

/// CSS `deepskyblue`, `#00bfff`
pub const DEEPSKYBLUE: Color = Color::linear(0.0, 0.520_995_56, 1.0);

/// CSS `dimgray`, `#696969`
pub const DIMGRAY: Color = Color::linear(0.141_263_29, 0.141_263_29, 0.141_263_29);

/// CSS `dimgrey`, `#696969`
pub const DIMGREY: Color = Color::linear(0.141_263_29, 0.141_263_29, 0.141_263_29);

/// CSS `dodgerblue`, `#1e90ff`
pub const DODGERBLUE: Color = Color::linear(0.012_983_032_5, 0.278_894_28, 1.0);

/// CSS `firebrick`, `#b22222`
pub const FIREBRICK: Color = Color::linear(0.445_201_2, 0.015_996_294, 0.015_996_294);

/// CSS `floralwhite`, `#fffaf0`
pub const FLORALWHITE: Color = Color::linear(1.0, 0.955_973_3, 0.871_367_1);

/// CSS `forestgreen`, `#228b22`
pub const FORESTGREEN: Color = Color::linear(0.015_996_294, 0.258_182_85, 0.015_996_294);

/// CSS `fuchsia`, `#ff00ff`
pub const FUCHSIA: Color = Color::linear(1.0, 0.0, 1.0);

/// CSS `gainsboro`, `#dcdcdc`
pub const GAINSBORO: Color = Color::linear(0.715_693_5, 0.715_693_5, 0.715_693_5);

/// CSS `ghostwhite`, `#f8f8ff`
pub const GHOSTWHITE: Color = Color::linear(0.938_685_7, 0.938_685_7, 1.0);

/// CSS `gold`, `#ffd700`
pub const GOLD: Color = Color::linear(1.0, 0.679_542_5, 0.0);

/// CSS `goldenrod`, `#daa520`
pub const GOLDENROD: Color = Color::linear(0.701_101_9, 0.376_262_13, 0.014_443_844);

/// CSS `gray`, `#808080`
pub const GRAY: Color = Color::linear(0.215_860_5, 0.215_860_5, 0.215_860_5);

/// CSS `green`, `#008000`
pub const GREEN: Color = Color::linear(0.0, 0.215_860_5, 0.0);

/// CSS `greenyellow`, `#adff2f`
pub const GREENYELLOW: Color = Color::linear(0.417_885_07, 1.0, 0.028_426_04);

/// CSS `grey`, `#808080`
pub const GREY: Color = Color::linear(0.215_860_5, 0.215_860_5, 0.215_860_5);

/// CSS `honeydew`, `#f0fff0`
pub const HONEYDEW: Color = Color::linear(0.871_367_1, 1.0, 0.871_367_1);

/// CSS `hotpink`, `#ff69b4`
pub const HOTPINK: Color = Color::linear(1.0, 0.141_263_29, 0.456_411_03);

/// CSS `indianred`, `#cd5c5c`
pub const INDIANRED: Color = Color::linear(0.610_495_57, 0.107_023_105, 0.107_023_105);

/// CSS `indigo`, `#4b0082`
pub const INDIGO: Color = Color::linear(0.070_360_094, 0.0, 0.223_227_96);

/// CSS `ivory`, `#fffff0`
pub const IVORY: Color = Color::linear(1.0, 1.0, 0.871_367_1);

/// CSS `khaki`, `#f0e68c`
pub const KHAKI: Color = Color::linear(0.871_367_1, 0.791_297_9, 0.262_250_66);

/// CSS `lavender`, `#e6e6fa`
pub const LAVENDER: Color = Color::linear(0.791_297_9, 0.791_297_9, 0.955_973_3);

/// CSS `lavenderblush`, `#fff0f5`
pub const LAVENDERBLUSH: Color = Color::linear(1.0, 0.871_367_1, 0.913_098_63);

/// CSS `lawngreen`, `#7cfc00`
pub const LAWNGREEN: Color = Color::linear(0.201_556_25, 0.973_445_3, 0.0);

/// CSS `lemonchiffon`, `#fffacd`
pub const LEMONCHIFFON: Color = Color::linear(1.0, 0.955_973_3, 0.610_495_57);

/// CSS `lightblue`, `#add8e6`
pub const LIGHTBLUE: Color = Color::linear(0.417_885_07, 0.686_685_3, 0.791_297_9);

/// CSS `lightcoral`, `#f08080`
pub const LIGHTCORAL: Color = Color::linear(0.871_367_1, 0.215_860_5, 0.215_860_5);

/// CSS `lightcyan`, `#e0ffff`
pub const LIGHTCYAN: Color = Color::linear(0.745_404_2, 1.0, 1.0);

/// CSS `lightgoldenrodyellow`, `#fafad2`
pub const LIGHTGOLDENRODYELLOW: Color = Color::linear(0.955_973_3, 0.955_973_3, 0.644_479_7);

/// CSS `lightgray`, `#d3d3d3`
pub const LIGHTGRAY: Color = Color::linear(0.651_405_63, 0.651_405_63, 0.651_405_63);

/// CSS `lightgreen`, `#90ee90`
pub const LIGHTGREEN: Color = Color::linear(0.278_894_28, 0.854_992_6, 0.278_894_28);

/// CSS `lightgrey`, `#d3d3d3`
pub const LIGHTGREY: Color = Color::linear(0.651_405_63, 0.651_405_63, 0.651_405_63);

/// CSS `lightpink`, `#ffb6c1`
pub const LIGHTPINK: Color = Color::linear(1.0, 0.467_783_8, 0.533_276_4);

/// CSS `lightsalmon`, `#ffa07a`
pub const LIGHTSALMON: Color = Color::linear(1.0, 0.351_532_6, 0.194_617_84);

/// CSS `lightseagreen`, `#20b2aa`
pub const LIGHTSEAGREEN: Color = Color::linear(0.014_443_844, 0.445_201_2, 0.401_977_78);

/// CSS `lightskyblue`, `#87cefa`
pub const LIGHTSKYBLUE: Color = Color::linear(0.242_281_12, 0.617_206_6, 0.955_973_3);

/// CSS `lightslategray`, `#778899`
pub const LIGHTSLATEGRAY: Color = Color::linear(0.184_474_99, 0.246_201_32, 0.318_546_77);

/// CSS `lightslategrey`, `#778899`
pub const LIGHTSLATEGREY: Color = Color::linear(0.184_474_99, 0.246_201_32, 0.318_546_77);

/// CSS `lightsteelblue`, `#b0c4de`
pub const LIGHTSTEELBLUE: Color = Color::linear(0.434_153_65, 0.552_011_43, 0.730_460_76);

/// CSS `lightyellow`, `#ffffe0`
pub const LIGHTYELLOW: Color = Color::linear(1.0, 1.0, 0.745_404_2);

/// CSS `lime`, `#00ff00`
pub const LIME: Color = Color::linear(0.0, 1.0, 0.0);

/// CSS `limegreen`, `#32cd32`
pub const LIMEGREEN: Color = Color::linear(0.031_896_032, 0.610_495_57, 0.031_896_032);

/// CSS `linen`, `#faf0e6`
pub const LINEN: Color = Color::linear(0.955_973_3, 0.871_367_1, 0.791_297_9);

/// CSS `magenta`, `#ff00ff`
pub const MAGENTA: Color = Color::linear(1.0, 0.0, 1.0);

/// CSS `maroon`, `#800000`
pub const MAROON: Color = Color::linear(0.215_860_5, 0.0, 0.0);

/// CSS `mediumaquamarine`, `#66cdaa`
pub const MEDIUMAQUAMARINE: Color = Color::linear(0.132_868_32, 0.610_495_57, 0.401_977_78);

/// CSS `mediumblue`, `#0000cd`
pub const MEDIUMBLUE: Color = Color::linear(0.0, 0.0, 0.610_495_57);

/// CSS `mediumorchid`, `#ba55d3`
pub const MEDIUMORCHID: Color = Color::linear(0.491_020_86, 0.090_841_71, 0.651_405_63);

/// CSS `mediumpurple`, `#9370db`
pub const MEDIUMPURPLE: Color = Color::linear(0.291_770_64, 0.162_029_37, 0.708_375_75);

/// CSS `mediumseagreen`, `#3cb371`
pub const MEDIUMSEAGREEN: Color = Color::linear(0.045_186_203, 0.450_785_8, 0.165_132_2);

/// CSS `mediumslateblue`, `#7b68ee`
pub const MEDIUMSLATEBLUE: Color = Color::linear(0.198_069_32, 0.138_431_61, 0.854_992_6);

/// CSS `mediumspringgreen`, `#00fa9a`
pub const MEDIUMSPRINGGREEN: Color = Color::linear(0.0, 0.955_973_3, 0.323_143_2);

/// CSS `mediumturquoise`, `#48d1cc`
pub const MEDIUMTURQUOISE: Color = Color::linear(0.064_803_265, 0.637_596_85, 0.603_827_36);

/// CSS `mediumvioletred`, `#c71585`
pub const MEDIUMVIOLETRED: Color = Color::linear(0.571_124_85, 0.007_499_032, 0.234_550_58);

/// CSS `midnightblue`, `#191970`
pub const MIDNIGHTBLUE: Color = Color::linear(0.009_721_218, 0.009_721_218, 0.162_029_37);

/// CSS `mintcream`, `#f5fffa`
pub const MINTCREAM: Color = Color::linear(0.913_098_63, 1.0, 0.955_973_3);

/// CSS `mistyrose`, `#ffe4e1`
pub const MISTYROSE: Color = Color::linear(1.0, 0.775_822_2, 0.752_942_2);

/// CSS `moccasin`, `#ffe4b5`
pub const MOCCASIN: Color = Color::linear(1.0, 0.775_822_2, 0.462_077);

/// CSS `navajowhite`, `#ffdead`
pub const NAVAJOWHITE: Color = Color::linear(1.0, 0.730_460_76, 0.417_885_07);

/// CSS `navy`, `#000080`
pub const NAVY: Color = Color::linear(0.0, 0.0, 0.215_860_5);

/// CSS `oldlace`, `#fdf5e6`
pub const OLDLACE: Color = Color::linear(0.982_250_6, 0.913_098_63, 0.791_297_9);

/// CSS `olive`, `#808000`
pub const OLIVE: Color = Color::linear(0.215_860_5, 0.215_860_5, 0.0);

/// CSS `olivedrab`, `#6b8e23`
pub const OLIVEDRAB: Color = Color::linear(0.147_027_27, 0.270_497_8, 0.016_807_375);

/// CSS `orange`, `#ffa500`
pub const ORANGE: Color = Color::linear(1.0, 0.376_262_13, 0.0);

/// CSS `orangered`, `#ff4500`
pub const ORANGERED: Color = Color::linear(1.0, 0.059_511_237, 0.0);

/// CSS `orchid`, `#da70d6`
pub const ORCHID: Color = Color::linear(0.701_101_9, 0.162_029_37, 0.672_443_15);

/// CSS `palegoldenrod`, `#eee8aa`
pub const PALEGOLDENROD: Color = Color::linear(0.854_992_6, 0.806_952_24, 0.401_977_78);

/// CSS `palegreen`, `#98fb98`
pub const PALEGREEN: Color = Color::linear(0.313_988_72, 0.964_686_3, 0.313_988_72);

/// CSS `paleturquoise`, `#afeeee`
pub const PALETURQUOISE: Color = Color::linear(0.428_690_5, 0.854_992_6, 0.854_992_6);

/// CSS `palevioletred`, `#db7093`
pub const PALEVIOLETRED: Color = Color::linear(0.708_375_75, 0.162_029_37, 0.291_770_64);

/// CSS `papayawhip`, `#ffefd5`
pub const PAPAYAWHIP: Color = Color::linear(1.0, 0.863_157_2, 0.665_387_3);

/// CSS `peachpuff`, `#ffdab9`
pub const PEACHPUFF: Color = Color::linear(1.0, 0.701_101_9, 0.485_149_95);

/// CSS `peru`, `#cd853f`
pub const PERU: Color = Color::linear(0.610_495_57, 0.234_550_58, 0.049_706_567);

/// CSS `pink`, `#ffc0cb`
pub const PINK: Color = Color::linear(1.0, 0.527_115_1, 0.597_201_76);

/// CSS `plum`, `#dda0dd`
pub const PLUM: Color = Color::linear(0.723_055_1, 0.351_532_6, 0.723_055_1);

/// CSS `powderblue`, `#b0e0e6`
pub const POWDERBLUE: Color = Color::linear(0.434_153_65, 0.745_404_2, 0.791_297_9);

/// CSS `purple`, `#800080`
pub const PURPLE: Color = Color::linear(0.215_860_5, 0.0, 0.215_860_5);

/// CSS `rebeccapurple`, `#663399`
pub const REBECCAPURPLE: Color = Color::linear(0.132_868_32, 0.033_104_766, 0.318_546_77);

/// CSS `red`, `#ff0000`
pub const RED: Color = Color::linear(1.0, 0.0, 0.0);

/// CSS `rosybrown`, `#bc8f8f`
pub const ROSYBROWN: Color = Color::linear(0.502_886_5, 0.274_677_3, 0.274_677_3);

/// CSS `royalblue`, `#4169e1`
pub const ROYALBLUE: Color = Color::linear(0.052_860_647, 0.141_263_29, 0.752_942_2);

/// CSS `saddlebrown`, `#8b4513`
pub const SADDLEBROWN: Color = Color::linear(0.258_182_85, 0.059_511_237, 0.006_512_090_6);

/// CSS `salmon`, `#fa8072`
pub const SALMON: Color = Color::linear(0.955_973_3, 0.215_860_5, 0.168_269_4);

/// CSS `sandybrown`, `#f4a460`
pub const SANDYBROWN: Color = Color::linear(0.904_661_2, 0.371_237_7, 0.116_970_666);

/// CSS `seagreen`, `#2e8b57`
pub const SEAGREEN: Color = Color::linear(0.027_320_892, 0.258_182_85, 0.095_307_47);

/// CSS `seashell`, `#fff5ee`
pub const SEASHELL: Color = Color::linear(1.0, 0.913_098_63, 0.854_992_6);

/// CSS `sienna`, `#a0522d`
pub const SIENNA: Color = Color::linear(0.351_532_6, 0.084_376_21, 0.026_241_222);

/// CSS `silver`, `#c0c0c0`
pub const SILVER: Color = Color::linear(0.527_115_1, 0.527_115_1, 0.527_115_1);

/// CSS `skyblue`, `#87ceeb`
pub const SKYBLUE: Color = Color::linear(0.242_281_12, 0.617_206_6, 0.830_769_9);

/// CSS `slateblue`, `#6a5acd`
pub const SLATEBLUE: Color = Color::linear(0.144_128_47, 0.102_241_73, 0.610_495_57);

/// CSS `slategray`, `#708090`
pub const SLATEGRAY: Color = Color::linear(0.162_029_37, 0.215_860_5, 0.278_894_28);

/// CSS `slategrey`, `#708090`
pub const SLATEGREY: Color = Color::linear(0.162_029_37, 0.215_860_5, 0.278_894_28);

/// CSS `snow`, `#fffafa`
pub const SNOW: Color = Color::linear(1.0, 0.955_973_3, 0.955_973_3);

/// CSS `springgreen`, `#00ff7f`
pub const SPRINGGREEN: Color = Color::linear(0.0, 1.0, 0.212_230_76);

/// CSS `steelblue`, `#4682b4`
pub const STEELBLUE: Color = Color::linear(0.061_246_052, 0.223_227_96, 0.456_411_03);

/// CSS `tan`, `#d2b48c`
pub const TAN: Color = Color::linear(0.644_479_7, 0.456_411_03, 0.262_250_66);

/// CSS `teal`, `#008080`
pub const TEAL: Color = Color::linear(0.0, 0.215_860_5, 0.215_860_5);

/// CSS `thistle`, `#d8bfd8`
pub const THISTLE: Color = Color::linear(0.686_685_3, 0.520_995_56, 0.686_685_3);

/// CSS `tomato`, `#ff6347`
pub const TOMATO: Color = Color::linear(1.0, 0.124_771_82, 0.063_010_015);

/// CSS `turquoise`, `#40e0d0`
pub const TURQUOISE: Color = Color::linear(0.051_269_457, 0.745_404_2, 0.630_757_15);

/// CSS `violet`, `#ee82ee`
pub const VIOLET: Color = Color::linear(0.854_992_6, 0.223_227_96, 0.854_992_6);

/// CSS `wheat`, `#f5deb3`
pub const WHEAT: Color = Color::linear(0.913_098_63, 0.730_460_76, 0.450_785_8);

/// CSS `white`, `#ffffff`
pub const WHITE: Color = Color::linear(1.0, 1.0, 1.0);

/// CSS `whitesmoke`, `#f5f5f5`
pub const WHITESMOKE: Color = Color::linear(0.913_098_63, 0.913_098_63, 0.913_098_63);

/// CSS `yellow`, `#ffff00`
pub const YELLOW: Color = Color::linear(1.0, 1.0, 0.0);

/// CSS `yellowgreen`, `#9acd32`
pub const YELLOWGREEN: Color = Color::linear(0.323_143_2, 0.610_495_57, 0.031_896_032);
