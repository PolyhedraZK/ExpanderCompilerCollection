use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proof::ComputationGraph;
use expander_compiler::zkcuda::proving_system::{ExpanderGKRProvingSystem, ParallelizedExpanderGKRProvingSystem, ProvingSystem,};
use expander_compiler::zkcuda::{context::*, kernel::*};
use gkr::BN254ConfigSha2Hyrax;
use gkr_engine::FieldEngine;
use serdes::ExpSerde;
use serde::{Deserialize, Serialize};
use std::fs;
struct Circuit {
	output: Vec<Vec<BN254Fr>>, 
	input: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_features_0_0_Conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_features_0_0_Conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_features_0_0_Conv_output_0_relu: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_features_0_0_Conv_output_0_min: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_features_0_2_PRelu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_1_conv_conv_0_conv_0_2_PRelu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_1_conv_conv_1_Conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_1_conv_conv_1_Conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_0_conv_0_2_PRelu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_relu: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_min: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_1_conv_1_2_PRelu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_2_Conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_conv_conv_2_Conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_relu: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_min: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_conv_conv_0_conv_0_2_PRelu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_621: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_622: Vec<BN254Fr>, 
	onnx__Conv_622_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_621_nscale: BN254Fr, 
	onnx__Conv_621_dscale: BN254Fr, 
	onnx__PRelu_779_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_779_nscale: BN254Fr, 
	onnx__PRelu_779_dscale: BN254Fr, 
	onnx__PRelu_779_zero: Vec<BN254Fr>, 
	onnx__Conv_624: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_625: Vec<BN254Fr>, 
	onnx__Conv_625_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_624_nscale: BN254Fr, 
	onnx__Conv_624_dscale: BN254Fr, 
	onnx__PRelu_780_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_780_nscale: BN254Fr, 
	onnx__PRelu_780_dscale: BN254Fr, 
	onnx__PRelu_780_zero: Vec<BN254Fr>, 
	onnx__Conv_627: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_628: Vec<BN254Fr>, 
	onnx__Conv_628_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_627_nscale: BN254Fr, 
	onnx__Conv_627_dscale: BN254Fr, 
	onnx__Conv_630: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_631: Vec<BN254Fr>, 
	onnx__Conv_631_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_630_nscale: BN254Fr, 
	onnx__Conv_630_dscale: BN254Fr, 
	onnx__PRelu_781_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_781_nscale: BN254Fr, 
	onnx__PRelu_781_dscale: BN254Fr, 
	onnx__PRelu_781_zero: Vec<BN254Fr>, 
	onnx__Conv_633: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_634: Vec<BN254Fr>, 
	onnx__Conv_634_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_633_nscale: BN254Fr, 
	onnx__Conv_633_dscale: BN254Fr, 
	onnx__PRelu_782_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_782_nscale: BN254Fr, 
	onnx__PRelu_782_dscale: BN254Fr, 
	onnx__PRelu_782_zero: Vec<BN254Fr>, 
	onnx__Conv_636: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_637: Vec<BN254Fr>, 
	onnx__Conv_637_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_636_nscale: BN254Fr, 
	onnx__Conv_636_dscale: BN254Fr, 
	onnx__Conv_639: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_640: Vec<BN254Fr>, 
	onnx__Conv_640_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_639_nscale: BN254Fr, 
	onnx__Conv_639_dscale: BN254Fr, 
	onnx__PRelu_783_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_783_nscale: BN254Fr, 
	onnx__PRelu_783_dscale: BN254Fr, 
	onnx__PRelu_783_zero: Vec<BN254Fr>, 
	onnx__Conv_642: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_643: Vec<BN254Fr>, 
	onnx__Conv_643_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_642_nscale: BN254Fr, 
	onnx__Conv_642_dscale: BN254Fr, 
	onnx__PRelu_784_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_784_nscale: BN254Fr, 
	onnx__PRelu_784_dscale: BN254Fr, 
	onnx__PRelu_784_zero: Vec<BN254Fr>, 
	onnx__Conv_645: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_646: Vec<BN254Fr>, 
	onnx__Conv_646_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_645_nscale: BN254Fr, 
	onnx__Conv_645_dscale: BN254Fr, 
	_features_features_3_Add_output_0_1nscale: BN254Fr, 
	_features_features_3_Add_output_0_1dscale: BN254Fr, 
	_features_features_3_Add_output_0_2nscale: BN254Fr, 
	_features_features_3_Add_output_0_2dscale: BN254Fr, 
	onnx__Conv_648: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_649: Vec<BN254Fr>, 
	onnx__Conv_649_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_648_nscale: BN254Fr, 
	onnx__Conv_648_dscale: BN254Fr, 
	onnx__PRelu_785_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_785_nscale: BN254Fr, 
	onnx__PRelu_785_dscale: BN254Fr, 
	onnx__PRelu_785_zero: Vec<BN254Fr>, 
	onnx__Conv_651: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_652: Vec<BN254Fr>, 
	onnx__Conv_652_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_651_nscale: BN254Fr, 
	onnx__Conv_651_dscale: BN254Fr, 
	onnx__PRelu_786_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_786_nscale: BN254Fr, 
	onnx__PRelu_786_dscale: BN254Fr, 
	onnx__PRelu_786_zero: Vec<BN254Fr>, 
	onnx__Conv_654: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_655: Vec<BN254Fr>, 
	onnx__Conv_655_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_654_nscale: BN254Fr, 
	onnx__Conv_654_dscale: BN254Fr, 
	onnx__Conv_657: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_658: Vec<BN254Fr>, 
	onnx__Conv_658_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_657_nscale: BN254Fr, 
	onnx__Conv_657_dscale: BN254Fr, 
	onnx__PRelu_787_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_787_nscale: BN254Fr, 
	onnx__PRelu_787_dscale: BN254Fr, 
	onnx__PRelu_787_zero: Vec<BN254Fr>, 
	onnx__Conv_660: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_661: Vec<BN254Fr>, 
	onnx__Conv_661_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_660_nscale: BN254Fr, 
	onnx__Conv_660_dscale: BN254Fr, 
	onnx__PRelu_788_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_788_nscale: BN254Fr, 
	onnx__PRelu_788_dscale: BN254Fr, 
	onnx__PRelu_788_zero: Vec<BN254Fr>, 
	onnx__Conv_663: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_664: Vec<BN254Fr>, 
	onnx__Conv_664_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_663_nscale: BN254Fr, 
	onnx__Conv_663_dscale: BN254Fr, 
	_features_features_5_Add_output_0_1nscale: BN254Fr, 
	_features_features_5_Add_output_0_1dscale: BN254Fr, 
	_features_features_5_Add_output_0_2nscale: BN254Fr, 
	_features_features_5_Add_output_0_2dscale: BN254Fr, 
	onnx__Conv_666: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_667: Vec<BN254Fr>, 
	onnx__Conv_667_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_666_nscale: BN254Fr, 
	onnx__Conv_666_dscale: BN254Fr, 
	onnx__PRelu_789_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_789_nscale: BN254Fr, 
	onnx__PRelu_789_dscale: BN254Fr, 
	onnx__PRelu_789_zero: Vec<BN254Fr>, 
	onnx__Conv_669: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_670: Vec<BN254Fr>, 
	onnx__Conv_670_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_669_nscale: BN254Fr, 
	onnx__Conv_669_dscale: BN254Fr, 
	onnx__PRelu_790_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_790_nscale: BN254Fr, 
	onnx__PRelu_790_dscale: BN254Fr, 
	onnx__PRelu_790_zero: Vec<BN254Fr>, 
	onnx__Conv_672: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_673: Vec<BN254Fr>, 
	onnx__Conv_673_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_672_nscale: BN254Fr, 
	onnx__Conv_672_dscale: BN254Fr, 
	_features_features_6_Add_output_0_1nscale: BN254Fr, 
	_features_features_6_Add_output_0_1dscale: BN254Fr, 
	_features_features_6_Add_output_0_2nscale: BN254Fr, 
	_features_features_6_Add_output_0_2dscale: BN254Fr, 
	onnx__Conv_675: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_676: Vec<BN254Fr>, 
	onnx__Conv_676_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_675_nscale: BN254Fr, 
	onnx__Conv_675_dscale: BN254Fr, 
	onnx__PRelu_791_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_791_nscale: BN254Fr, 
	onnx__PRelu_791_dscale: BN254Fr, 
	onnx__PRelu_791_zero: Vec<BN254Fr>, 
	onnx__Conv_678: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_679: Vec<BN254Fr>, 
	onnx__Conv_679_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_678_nscale: BN254Fr, 
	onnx__Conv_678_dscale: BN254Fr, 
	onnx__PRelu_792_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_792_nscale: BN254Fr, 
	onnx__PRelu_792_dscale: BN254Fr, 
	onnx__PRelu_792_zero: Vec<BN254Fr>, 
	onnx__Conv_681: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_682: Vec<BN254Fr>, 
	onnx__Conv_682_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_681_nscale: BN254Fr, 
	onnx__Conv_681_dscale: BN254Fr, 
	onnx__Conv_684: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_685: Vec<BN254Fr>, 
	onnx__Conv_685_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_684_nscale: BN254Fr, 
	onnx__Conv_684_dscale: BN254Fr, 
	onnx__PRelu_793_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_793_nscale: BN254Fr, 
	onnx__PRelu_793_dscale: BN254Fr, 
	onnx__PRelu_793_zero: Vec<BN254Fr>, 
	onnx__Conv_687: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_688: Vec<BN254Fr>, 
	onnx__Conv_688_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_687_nscale: BN254Fr, 
	onnx__Conv_687_dscale: BN254Fr, 
	onnx__PRelu_794_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_794_nscale: BN254Fr, 
	onnx__PRelu_794_dscale: BN254Fr, 
	onnx__PRelu_794_zero: Vec<BN254Fr>, 
	onnx__Conv_690: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_691: Vec<BN254Fr>, 
	onnx__Conv_691_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_690_nscale: BN254Fr, 
	onnx__Conv_690_dscale: BN254Fr, 
	_features_features_8_Add_output_0_1nscale: BN254Fr, 
	_features_features_8_Add_output_0_1dscale: BN254Fr, 
	_features_features_8_Add_output_0_2nscale: BN254Fr, 
	_features_features_8_Add_output_0_2dscale: BN254Fr, 
	onnx__Conv_693: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_694: Vec<BN254Fr>, 
	onnx__Conv_694_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_693_nscale: BN254Fr, 
	onnx__Conv_693_dscale: BN254Fr, 
	onnx__PRelu_795_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_795_nscale: BN254Fr, 
	onnx__PRelu_795_dscale: BN254Fr, 
	onnx__PRelu_795_zero: Vec<BN254Fr>, 
	onnx__Conv_696: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_697: Vec<BN254Fr>, 
	onnx__Conv_697_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_696_nscale: BN254Fr, 
	onnx__Conv_696_dscale: BN254Fr, 
	onnx__PRelu_796_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_796_nscale: BN254Fr, 
	onnx__PRelu_796_dscale: BN254Fr, 
	onnx__PRelu_796_zero: Vec<BN254Fr>, 
	onnx__Conv_699: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_700: Vec<BN254Fr>, 
	onnx__Conv_700_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_699_nscale: BN254Fr, 
	onnx__Conv_699_dscale: BN254Fr, 
	_features_features_9_Add_output_0_1nscale: BN254Fr, 
	_features_features_9_Add_output_0_1dscale: BN254Fr, 
	_features_features_9_Add_output_0_2nscale: BN254Fr, 
	_features_features_9_Add_output_0_2dscale: BN254Fr, 
	onnx__Conv_702: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_703: Vec<BN254Fr>, 
	onnx__Conv_703_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_702_nscale: BN254Fr, 
	onnx__Conv_702_dscale: BN254Fr, 
	onnx__PRelu_797_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_797_nscale: BN254Fr, 
	onnx__PRelu_797_dscale: BN254Fr, 
	onnx__PRelu_797_zero: Vec<BN254Fr>, 
	onnx__Conv_705: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_706: Vec<BN254Fr>, 
	onnx__Conv_706_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_705_nscale: BN254Fr, 
	onnx__Conv_705_dscale: BN254Fr, 
	onnx__PRelu_798_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_798_nscale: BN254Fr, 
	onnx__PRelu_798_dscale: BN254Fr, 
	onnx__PRelu_798_zero: Vec<BN254Fr>, 
	onnx__Conv_708: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_709: Vec<BN254Fr>, 
	onnx__Conv_709_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_708_nscale: BN254Fr, 
	onnx__Conv_708_dscale: BN254Fr, 
	_features_features_10_Add_output_0_1nscale: BN254Fr, 
	_features_features_10_Add_output_0_1dscale: BN254Fr, 
	_features_features_10_Add_output_0_2nscale: BN254Fr, 
	_features_features_10_Add_output_0_2dscale: BN254Fr, 
	onnx__Conv_711: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_712: Vec<BN254Fr>, 
	onnx__Conv_712_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_711_nscale: BN254Fr, 
	onnx__Conv_711_dscale: BN254Fr, 
	onnx__PRelu_799_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_799_nscale: BN254Fr, 
	onnx__PRelu_799_dscale: BN254Fr, 
	onnx__PRelu_799_zero: Vec<BN254Fr>, 
	onnx__Conv_714: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_715: Vec<BN254Fr>, 
	onnx__Conv_715_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_714_nscale: BN254Fr, 
	onnx__Conv_714_dscale: BN254Fr, 
	onnx__PRelu_800_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_800_nscale: BN254Fr, 
	onnx__PRelu_800_dscale: BN254Fr, 
	onnx__PRelu_800_zero: Vec<BN254Fr>, 
	onnx__Conv_717: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_718: Vec<BN254Fr>, 
	onnx__Conv_718_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_717_nscale: BN254Fr, 
	onnx__Conv_717_dscale: BN254Fr, 
	onnx__Conv_720: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_721: Vec<BN254Fr>, 
	onnx__Conv_721_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_720_nscale: BN254Fr, 
	onnx__Conv_720_dscale: BN254Fr, 
	onnx__PRelu_801_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_801_nscale: BN254Fr, 
	onnx__PRelu_801_dscale: BN254Fr, 
	onnx__PRelu_801_zero: Vec<BN254Fr>, 
	onnx__Conv_723: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_724: Vec<BN254Fr>, 
	onnx__Conv_724_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_723_nscale: BN254Fr, 
	onnx__Conv_723_dscale: BN254Fr, 
	onnx__PRelu_802_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_802_nscale: BN254Fr, 
	onnx__PRelu_802_dscale: BN254Fr, 
	onnx__PRelu_802_zero: Vec<BN254Fr>, 
	onnx__Conv_726: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_727: Vec<BN254Fr>, 
	onnx__Conv_727_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_726_nscale: BN254Fr, 
	onnx__Conv_726_dscale: BN254Fr, 
	_features_features_12_Add_output_0_1nscale: BN254Fr, 
	_features_features_12_Add_output_0_1dscale: BN254Fr, 
	_features_features_12_Add_output_0_2nscale: BN254Fr, 
	_features_features_12_Add_output_0_2dscale: BN254Fr, 
	onnx__Conv_729: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_730: Vec<BN254Fr>, 
	onnx__Conv_730_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_729_nscale: BN254Fr, 
	onnx__Conv_729_dscale: BN254Fr, 
	onnx__PRelu_803_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_803_nscale: BN254Fr, 
	onnx__PRelu_803_dscale: BN254Fr, 
	onnx__PRelu_803_zero: Vec<BN254Fr>, 
	onnx__Conv_732: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_733: Vec<BN254Fr>, 
	onnx__Conv_733_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_732_nscale: BN254Fr, 
	onnx__Conv_732_dscale: BN254Fr, 
	onnx__PRelu_804_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_804_nscale: BN254Fr, 
	onnx__PRelu_804_dscale: BN254Fr, 
	onnx__PRelu_804_zero: Vec<BN254Fr>, 
	onnx__Conv_735: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_736: Vec<BN254Fr>, 
	onnx__Conv_736_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_735_nscale: BN254Fr, 
	onnx__Conv_735_dscale: BN254Fr, 
	_features_features_13_Add_output_0_1nscale: BN254Fr, 
	_features_features_13_Add_output_0_1dscale: BN254Fr, 
	_features_features_13_Add_output_0_2nscale: BN254Fr, 
	_features_features_13_Add_output_0_2dscale: BN254Fr, 
	onnx__Conv_738: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_739: Vec<BN254Fr>, 
	onnx__Conv_739_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_738_nscale: BN254Fr, 
	onnx__Conv_738_dscale: BN254Fr, 
	onnx__PRelu_805_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_805_nscale: BN254Fr, 
	onnx__PRelu_805_dscale: BN254Fr, 
	onnx__PRelu_805_zero: Vec<BN254Fr>, 
	onnx__Conv_741: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_742: Vec<BN254Fr>, 
	onnx__Conv_742_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_741_nscale: BN254Fr, 
	onnx__Conv_741_dscale: BN254Fr, 
	onnx__PRelu_806_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_806_nscale: BN254Fr, 
	onnx__PRelu_806_dscale: BN254Fr, 
	onnx__PRelu_806_zero: Vec<BN254Fr>, 
	onnx__Conv_744: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_745: Vec<BN254Fr>, 
	onnx__Conv_745_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_744_nscale: BN254Fr, 
	onnx__Conv_744_dscale: BN254Fr, 
	onnx__Conv_747: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_748: Vec<BN254Fr>, 
	onnx__Conv_748_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_747_nscale: BN254Fr, 
	onnx__Conv_747_dscale: BN254Fr, 
	onnx__PRelu_807_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_807_nscale: BN254Fr, 
	onnx__PRelu_807_dscale: BN254Fr, 
	onnx__PRelu_807_zero: Vec<BN254Fr>, 
	onnx__Conv_750: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_751: Vec<BN254Fr>, 
	onnx__Conv_751_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_750_nscale: BN254Fr, 
	onnx__Conv_750_dscale: BN254Fr, 
	onnx__PRelu_808_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_808_nscale: BN254Fr, 
	onnx__PRelu_808_dscale: BN254Fr, 
	onnx__PRelu_808_zero: Vec<BN254Fr>, 
	onnx__Conv_753: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_754: Vec<BN254Fr>, 
	onnx__Conv_754_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_753_nscale: BN254Fr, 
	onnx__Conv_753_dscale: BN254Fr, 
	_features_features_15_Add_output_0_1nscale: BN254Fr, 
	_features_features_15_Add_output_0_1dscale: BN254Fr, 
	_features_features_15_Add_output_0_2nscale: BN254Fr, 
	_features_features_15_Add_output_0_2dscale: BN254Fr, 
	onnx__Conv_756: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_757: Vec<BN254Fr>, 
	onnx__Conv_757_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_756_nscale: BN254Fr, 
	onnx__Conv_756_dscale: BN254Fr, 
	onnx__PRelu_809_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_809_nscale: BN254Fr, 
	onnx__PRelu_809_dscale: BN254Fr, 
	onnx__PRelu_809_zero: Vec<BN254Fr>, 
	onnx__Conv_759: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_760: Vec<BN254Fr>, 
	onnx__Conv_760_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_759_nscale: BN254Fr, 
	onnx__Conv_759_dscale: BN254Fr, 
	onnx__PRelu_810_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_810_nscale: BN254Fr, 
	onnx__PRelu_810_dscale: BN254Fr, 
	onnx__PRelu_810_zero: Vec<BN254Fr>, 
	onnx__Conv_762: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_763: Vec<BN254Fr>, 
	onnx__Conv_763_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_762_nscale: BN254Fr, 
	onnx__Conv_762_dscale: BN254Fr, 
	_features_features_16_Add_output_0_1nscale: BN254Fr, 
	_features_features_16_Add_output_0_1dscale: BN254Fr, 
	_features_features_16_Add_output_0_2nscale: BN254Fr, 
	_features_features_16_Add_output_0_2dscale: BN254Fr, 
	onnx__Conv_765: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_766: Vec<BN254Fr>, 
	onnx__Conv_766_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_765_nscale: BN254Fr, 
	onnx__Conv_765_dscale: BN254Fr, 
	onnx__PRelu_811_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_811_nscale: BN254Fr, 
	onnx__PRelu_811_dscale: BN254Fr, 
	onnx__PRelu_811_zero: Vec<BN254Fr>, 
	onnx__Conv_768: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_769: Vec<BN254Fr>, 
	onnx__Conv_769_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_768_nscale: BN254Fr, 
	onnx__Conv_768_dscale: BN254Fr, 
	onnx__PRelu_812_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_812_nscale: BN254Fr, 
	onnx__PRelu_812_dscale: BN254Fr, 
	onnx__PRelu_812_zero: Vec<BN254Fr>, 
	onnx__Conv_771: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_772: Vec<BN254Fr>, 
	onnx__Conv_772_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_771_nscale: BN254Fr, 
	onnx__Conv_771_dscale: BN254Fr, 
	onnx__Conv_774: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_775: Vec<BN254Fr>, 
	onnx__Conv_775_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_774_nscale: BN254Fr, 
	onnx__Conv_774_dscale: BN254Fr, 
	onnx__PRelu_813_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__PRelu_813_nscale: BN254Fr, 
	onnx__PRelu_813_dscale: BN254Fr, 
	onnx__PRelu_813_zero: Vec<BN254Fr>, 
	onnx__Conv_777: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__Conv_778: Vec<BN254Fr>, 
	onnx__Conv_778_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx__Conv_777_nscale: BN254Fr, 
	onnx__Conv_777_dscale: BN254Fr, 
	onnx__MatMul_814: Vec<Vec<BN254Fr>>, 
	onnx__MatMul_814_nscale: BN254Fr, 
	onnx__MatMul_814_dscale: BN254Fr, 
}

#[derive(Serialize, Deserialize, Debug)]
struct Circuit_Input {
	output: Vec<Vec<i64>>, 
	input: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_features_0_0_Conv_output_0_conv: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_features_0_0_Conv_output_0_floor: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_features_0_0_Conv_output_0_relu: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_features_0_0_Conv_output_0_min: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_features_0_2_PRelu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_1_conv_conv_0_conv_0_2_PRelu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_1_conv_conv_1_Conv_output_0_conv: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_1_conv_conv_1_Conv_output_0_floor: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_0_conv_0_2_PRelu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_conv: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_floor: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_relu: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_min: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_1_conv_1_2_PRelu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_2_Conv_output_0_conv: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_2_conv_conv_2_Conv_output_0_floor: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_conv: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_floor: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_relu: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_min: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_conv_conv_0_conv_0_2_PRelu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_621: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_622: Vec<i64>, 
	onnx__Conv_622_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_621_nscale: i64, 
	onnx__Conv_621_dscale: i64, 
	onnx__PRelu_779_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_779_nscale: i64, 
	onnx__PRelu_779_dscale: i64, 
	onnx__PRelu_779_zero: Vec<i64>, 
	onnx__Conv_624: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_625: Vec<i64>, 
	onnx__Conv_625_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_624_nscale: i64, 
	onnx__Conv_624_dscale: i64, 
	onnx__PRelu_780_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_780_nscale: i64, 
	onnx__PRelu_780_dscale: i64, 
	onnx__PRelu_780_zero: Vec<i64>, 
	onnx__Conv_627: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_628: Vec<i64>, 
	onnx__Conv_628_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_627_nscale: i64, 
	onnx__Conv_627_dscale: i64, 
	onnx__Conv_630: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_631: Vec<i64>, 
	onnx__Conv_631_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_630_nscale: i64, 
	onnx__Conv_630_dscale: i64, 
	onnx__PRelu_781_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_781_nscale: i64, 
	onnx__PRelu_781_dscale: i64, 
	onnx__PRelu_781_zero: Vec<i64>, 
	onnx__Conv_633: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_634: Vec<i64>, 
	onnx__Conv_634_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_633_nscale: i64, 
	onnx__Conv_633_dscale: i64, 
	onnx__PRelu_782_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_782_nscale: i64, 
	onnx__PRelu_782_dscale: i64, 
	onnx__PRelu_782_zero: Vec<i64>, 
	onnx__Conv_636: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_637: Vec<i64>, 
	onnx__Conv_637_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_636_nscale: i64, 
	onnx__Conv_636_dscale: i64, 
	onnx__Conv_639: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_640: Vec<i64>, 
	onnx__Conv_640_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_639_nscale: i64, 
	onnx__Conv_639_dscale: i64, 
	onnx__PRelu_783_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_783_nscale: i64, 
	onnx__PRelu_783_dscale: i64, 
	onnx__PRelu_783_zero: Vec<i64>, 
	onnx__Conv_642: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_643: Vec<i64>, 
	onnx__Conv_643_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_642_nscale: i64, 
	onnx__Conv_642_dscale: i64, 
	onnx__PRelu_784_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_784_nscale: i64, 
	onnx__PRelu_784_dscale: i64, 
	onnx__PRelu_784_zero: Vec<i64>, 
	onnx__Conv_645: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_646: Vec<i64>, 
	onnx__Conv_646_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_645_nscale: i64, 
	onnx__Conv_645_dscale: i64, 
	_features_features_3_Add_output_0_1nscale: i64, 
	_features_features_3_Add_output_0_1dscale: i64, 
	_features_features_3_Add_output_0_2nscale: i64, 
	_features_features_3_Add_output_0_2dscale: i64, 
	onnx__Conv_648: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_649: Vec<i64>, 
	onnx__Conv_649_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_648_nscale: i64, 
	onnx__Conv_648_dscale: i64, 
	onnx__PRelu_785_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_785_nscale: i64, 
	onnx__PRelu_785_dscale: i64, 
	onnx__PRelu_785_zero: Vec<i64>, 
	onnx__Conv_651: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_652: Vec<i64>, 
	onnx__Conv_652_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_651_nscale: i64, 
	onnx__Conv_651_dscale: i64, 
	onnx__PRelu_786_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_786_nscale: i64, 
	onnx__PRelu_786_dscale: i64, 
	onnx__PRelu_786_zero: Vec<i64>, 
	onnx__Conv_654: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_655: Vec<i64>, 
	onnx__Conv_655_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_654_nscale: i64, 
	onnx__Conv_654_dscale: i64, 
	onnx__Conv_657: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_658: Vec<i64>, 
	onnx__Conv_658_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_657_nscale: i64, 
	onnx__Conv_657_dscale: i64, 
	onnx__PRelu_787_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_787_nscale: i64, 
	onnx__PRelu_787_dscale: i64, 
	onnx__PRelu_787_zero: Vec<i64>, 
	onnx__Conv_660: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_661: Vec<i64>, 
	onnx__Conv_661_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_660_nscale: i64, 
	onnx__Conv_660_dscale: i64, 
	onnx__PRelu_788_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_788_nscale: i64, 
	onnx__PRelu_788_dscale: i64, 
	onnx__PRelu_788_zero: Vec<i64>, 
	onnx__Conv_663: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_664: Vec<i64>, 
	onnx__Conv_664_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_663_nscale: i64, 
	onnx__Conv_663_dscale: i64, 
	_features_features_5_Add_output_0_1nscale: i64, 
	_features_features_5_Add_output_0_1dscale: i64, 
	_features_features_5_Add_output_0_2nscale: i64, 
	_features_features_5_Add_output_0_2dscale: i64, 
	onnx__Conv_666: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_667: Vec<i64>, 
	onnx__Conv_667_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_666_nscale: i64, 
	onnx__Conv_666_dscale: i64, 
	onnx__PRelu_789_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_789_nscale: i64, 
	onnx__PRelu_789_dscale: i64, 
	onnx__PRelu_789_zero: Vec<i64>, 
	onnx__Conv_669: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_670: Vec<i64>, 
	onnx__Conv_670_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_669_nscale: i64, 
	onnx__Conv_669_dscale: i64, 
	onnx__PRelu_790_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_790_nscale: i64, 
	onnx__PRelu_790_dscale: i64, 
	onnx__PRelu_790_zero: Vec<i64>, 
	onnx__Conv_672: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_673: Vec<i64>, 
	onnx__Conv_673_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_672_nscale: i64, 
	onnx__Conv_672_dscale: i64, 
	_features_features_6_Add_output_0_1nscale: i64, 
	_features_features_6_Add_output_0_1dscale: i64, 
	_features_features_6_Add_output_0_2nscale: i64, 
	_features_features_6_Add_output_0_2dscale: i64, 
	onnx__Conv_675: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_676: Vec<i64>, 
	onnx__Conv_676_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_675_nscale: i64, 
	onnx__Conv_675_dscale: i64, 
	onnx__PRelu_791_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_791_nscale: i64, 
	onnx__PRelu_791_dscale: i64, 
	onnx__PRelu_791_zero: Vec<i64>, 
	onnx__Conv_678: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_679: Vec<i64>, 
	onnx__Conv_679_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_678_nscale: i64, 
	onnx__Conv_678_dscale: i64, 
	onnx__PRelu_792_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_792_nscale: i64, 
	onnx__PRelu_792_dscale: i64, 
	onnx__PRelu_792_zero: Vec<i64>, 
	onnx__Conv_681: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_682: Vec<i64>, 
	onnx__Conv_682_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_681_nscale: i64, 
	onnx__Conv_681_dscale: i64, 
	onnx__Conv_684: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_685: Vec<i64>, 
	onnx__Conv_685_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_684_nscale: i64, 
	onnx__Conv_684_dscale: i64, 
	onnx__PRelu_793_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_793_nscale: i64, 
	onnx__PRelu_793_dscale: i64, 
	onnx__PRelu_793_zero: Vec<i64>, 
	onnx__Conv_687: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_688: Vec<i64>, 
	onnx__Conv_688_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_687_nscale: i64, 
	onnx__Conv_687_dscale: i64, 
	onnx__PRelu_794_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_794_nscale: i64, 
	onnx__PRelu_794_dscale: i64, 
	onnx__PRelu_794_zero: Vec<i64>, 
	onnx__Conv_690: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_691: Vec<i64>, 
	onnx__Conv_691_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_690_nscale: i64, 
	onnx__Conv_690_dscale: i64, 
	_features_features_8_Add_output_0_1nscale: i64, 
	_features_features_8_Add_output_0_1dscale: i64, 
	_features_features_8_Add_output_0_2nscale: i64, 
	_features_features_8_Add_output_0_2dscale: i64, 
	onnx__Conv_693: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_694: Vec<i64>, 
	onnx__Conv_694_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_693_nscale: i64, 
	onnx__Conv_693_dscale: i64, 
	onnx__PRelu_795_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_795_nscale: i64, 
	onnx__PRelu_795_dscale: i64, 
	onnx__PRelu_795_zero: Vec<i64>, 
	onnx__Conv_696: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_697: Vec<i64>, 
	onnx__Conv_697_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_696_nscale: i64, 
	onnx__Conv_696_dscale: i64, 
	onnx__PRelu_796_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_796_nscale: i64, 
	onnx__PRelu_796_dscale: i64, 
	onnx__PRelu_796_zero: Vec<i64>, 
	onnx__Conv_699: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_700: Vec<i64>, 
	onnx__Conv_700_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_699_nscale: i64, 
	onnx__Conv_699_dscale: i64, 
	_features_features_9_Add_output_0_1nscale: i64, 
	_features_features_9_Add_output_0_1dscale: i64, 
	_features_features_9_Add_output_0_2nscale: i64, 
	_features_features_9_Add_output_0_2dscale: i64, 
	onnx__Conv_702: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_703: Vec<i64>, 
	onnx__Conv_703_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_702_nscale: i64, 
	onnx__Conv_702_dscale: i64, 
	onnx__PRelu_797_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_797_nscale: i64, 
	onnx__PRelu_797_dscale: i64, 
	onnx__PRelu_797_zero: Vec<i64>, 
	onnx__Conv_705: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_706: Vec<i64>, 
	onnx__Conv_706_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_705_nscale: i64, 
	onnx__Conv_705_dscale: i64, 
	onnx__PRelu_798_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_798_nscale: i64, 
	onnx__PRelu_798_dscale: i64, 
	onnx__PRelu_798_zero: Vec<i64>, 
	onnx__Conv_708: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_709: Vec<i64>, 
	onnx__Conv_709_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_708_nscale: i64, 
	onnx__Conv_708_dscale: i64, 
	_features_features_10_Add_output_0_1nscale: i64, 
	_features_features_10_Add_output_0_1dscale: i64, 
	_features_features_10_Add_output_0_2nscale: i64, 
	_features_features_10_Add_output_0_2dscale: i64, 
	onnx__Conv_711: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_712: Vec<i64>, 
	onnx__Conv_712_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_711_nscale: i64, 
	onnx__Conv_711_dscale: i64, 
	onnx__PRelu_799_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_799_nscale: i64, 
	onnx__PRelu_799_dscale: i64, 
	onnx__PRelu_799_zero: Vec<i64>, 
	onnx__Conv_714: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_715: Vec<i64>, 
	onnx__Conv_715_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_714_nscale: i64, 
	onnx__Conv_714_dscale: i64, 
	onnx__PRelu_800_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_800_nscale: i64, 
	onnx__PRelu_800_dscale: i64, 
	onnx__PRelu_800_zero: Vec<i64>, 
	onnx__Conv_717: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_718: Vec<i64>, 
	onnx__Conv_718_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_717_nscale: i64, 
	onnx__Conv_717_dscale: i64, 
	onnx__Conv_720: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_721: Vec<i64>, 
	onnx__Conv_721_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_720_nscale: i64, 
	onnx__Conv_720_dscale: i64, 
	onnx__PRelu_801_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_801_nscale: i64, 
	onnx__PRelu_801_dscale: i64, 
	onnx__PRelu_801_zero: Vec<i64>, 
	onnx__Conv_723: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_724: Vec<i64>, 
	onnx__Conv_724_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_723_nscale: i64, 
	onnx__Conv_723_dscale: i64, 
	onnx__PRelu_802_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_802_nscale: i64, 
	onnx__PRelu_802_dscale: i64, 
	onnx__PRelu_802_zero: Vec<i64>, 
	onnx__Conv_726: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_727: Vec<i64>, 
	onnx__Conv_727_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_726_nscale: i64, 
	onnx__Conv_726_dscale: i64, 
	_features_features_12_Add_output_0_1nscale: i64, 
	_features_features_12_Add_output_0_1dscale: i64, 
	_features_features_12_Add_output_0_2nscale: i64, 
	_features_features_12_Add_output_0_2dscale: i64, 
	onnx__Conv_729: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_730: Vec<i64>, 
	onnx__Conv_730_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_729_nscale: i64, 
	onnx__Conv_729_dscale: i64, 
	onnx__PRelu_803_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_803_nscale: i64, 
	onnx__PRelu_803_dscale: i64, 
	onnx__PRelu_803_zero: Vec<i64>, 
	onnx__Conv_732: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_733: Vec<i64>, 
	onnx__Conv_733_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_732_nscale: i64, 
	onnx__Conv_732_dscale: i64, 
	onnx__PRelu_804_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_804_nscale: i64, 
	onnx__PRelu_804_dscale: i64, 
	onnx__PRelu_804_zero: Vec<i64>, 
	onnx__Conv_735: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_736: Vec<i64>, 
	onnx__Conv_736_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_735_nscale: i64, 
	onnx__Conv_735_dscale: i64, 
	_features_features_13_Add_output_0_1nscale: i64, 
	_features_features_13_Add_output_0_1dscale: i64, 
	_features_features_13_Add_output_0_2nscale: i64, 
	_features_features_13_Add_output_0_2dscale: i64, 
	onnx__Conv_738: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_739: Vec<i64>, 
	onnx__Conv_739_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_738_nscale: i64, 
	onnx__Conv_738_dscale: i64, 
	onnx__PRelu_805_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_805_nscale: i64, 
	onnx__PRelu_805_dscale: i64, 
	onnx__PRelu_805_zero: Vec<i64>, 
	onnx__Conv_741: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_742: Vec<i64>, 
	onnx__Conv_742_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_741_nscale: i64, 
	onnx__Conv_741_dscale: i64, 
	onnx__PRelu_806_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_806_nscale: i64, 
	onnx__PRelu_806_dscale: i64, 
	onnx__PRelu_806_zero: Vec<i64>, 
	onnx__Conv_744: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_745: Vec<i64>, 
	onnx__Conv_745_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_744_nscale: i64, 
	onnx__Conv_744_dscale: i64, 
	onnx__Conv_747: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_748: Vec<i64>, 
	onnx__Conv_748_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_747_nscale: i64, 
	onnx__Conv_747_dscale: i64, 
	onnx__PRelu_807_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_807_nscale: i64, 
	onnx__PRelu_807_dscale: i64, 
	onnx__PRelu_807_zero: Vec<i64>, 
	onnx__Conv_750: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_751: Vec<i64>, 
	onnx__Conv_751_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_750_nscale: i64, 
	onnx__Conv_750_dscale: i64, 
	onnx__PRelu_808_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_808_nscale: i64, 
	onnx__PRelu_808_dscale: i64, 
	onnx__PRelu_808_zero: Vec<i64>, 
	onnx__Conv_753: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_754: Vec<i64>, 
	onnx__Conv_754_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_753_nscale: i64, 
	onnx__Conv_753_dscale: i64, 
	_features_features_15_Add_output_0_1nscale: i64, 
	_features_features_15_Add_output_0_1dscale: i64, 
	_features_features_15_Add_output_0_2nscale: i64, 
	_features_features_15_Add_output_0_2dscale: i64, 
	onnx__Conv_756: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_757: Vec<i64>, 
	onnx__Conv_757_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_756_nscale: i64, 
	onnx__Conv_756_dscale: i64, 
	onnx__PRelu_809_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_809_nscale: i64, 
	onnx__PRelu_809_dscale: i64, 
	onnx__PRelu_809_zero: Vec<i64>, 
	onnx__Conv_759: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_760: Vec<i64>, 
	onnx__Conv_760_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_759_nscale: i64, 
	onnx__Conv_759_dscale: i64, 
	onnx__PRelu_810_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_810_nscale: i64, 
	onnx__PRelu_810_dscale: i64, 
	onnx__PRelu_810_zero: Vec<i64>, 
	onnx__Conv_762: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_763: Vec<i64>, 
	onnx__Conv_763_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_762_nscale: i64, 
	onnx__Conv_762_dscale: i64, 
	_features_features_16_Add_output_0_1nscale: i64, 
	_features_features_16_Add_output_0_1dscale: i64, 
	_features_features_16_Add_output_0_2nscale: i64, 
	_features_features_16_Add_output_0_2dscale: i64, 
	onnx__Conv_765: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_766: Vec<i64>, 
	onnx__Conv_766_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_765_nscale: i64, 
	onnx__Conv_765_dscale: i64, 
	onnx__PRelu_811_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_811_nscale: i64, 
	onnx__PRelu_811_dscale: i64, 
	onnx__PRelu_811_zero: Vec<i64>, 
	onnx__Conv_768: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_769: Vec<i64>, 
	onnx__Conv_769_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_768_nscale: i64, 
	onnx__Conv_768_dscale: i64, 
	onnx__PRelu_812_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_812_nscale: i64, 
	onnx__PRelu_812_dscale: i64, 
	onnx__PRelu_812_zero: Vec<i64>, 
	onnx__Conv_771: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_772: Vec<i64>, 
	onnx__Conv_772_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_771_nscale: i64, 
	onnx__Conv_771_dscale: i64, 
	onnx__Conv_774: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_775: Vec<i64>, 
	onnx__Conv_775_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_774_nscale: i64, 
	onnx__Conv_774_dscale: i64, 
	onnx__PRelu_813_q: Vec<Vec<Vec<i64>>>, 
	onnx__PRelu_813_nscale: i64, 
	onnx__PRelu_813_dscale: i64, 
	onnx__PRelu_813_zero: Vec<i64>, 
	onnx__Conv_777: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__Conv_778: Vec<i64>, 
	onnx__Conv_778_q: Vec<Vec<Vec<i64>>>, 
	onnx__Conv_777_nscale: i64, 
	onnx__Conv_777_dscale: i64, 
	onnx__MatMul_814: Vec<Vec<i64>>, 
	onnx__MatMul_814_nscale: i64, 
	onnx__MatMul_814_dscale: i64, 
}

fn input_copy(i_input: &Circuit_Input) -> Circuit{
	let mut output = vec![vec![BN254Fr::default();512];4]; 
	for i in 0..4 {
		for j in 0..512 {
			if i_input.output[i][j] >= 0{
				output[i][j] = BN254Fr::from((i_input.output[i][j]) as u64); 
			} else {
				output[i][j] = -BN254Fr::from((-i_input.output[i][j]) as u64); 
			} 
		}
	}
	let mut input = vec![vec![vec![vec![BN254Fr::default();112];112];3];4]; 
	for i in 0..4 {
		for j in 0..3 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input.input[i][j][k][l] >= 0{
						input[i][j][k][l] = BN254Fr::from((i_input.input[i][j][k][l]) as u64); 
					} else {
						input[i][j][k][l] = -BN254Fr::from((-i_input.input[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_0_features_0_0_Conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();112];112];32];4]; 
	for i in 0..4 {
		for j in 0..32 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_0_features_0_0_Conv_output_0_conv[i][j][k][l] >= 0{
						_features_features_0_features_0_0_Conv_output_0_conv[i][j][k][l] = BN254Fr::from((i_input._features_features_0_features_0_0_Conv_output_0_conv[i][j][k][l]) as u64); 
					} else {
						_features_features_0_features_0_0_Conv_output_0_conv[i][j][k][l] = -BN254Fr::from((-i_input._features_features_0_features_0_0_Conv_output_0_conv[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_0_features_0_0_Conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();112];112];32];4]; 
	for i in 0..4 {
		for j in 0..32 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_0_features_0_0_Conv_output_0_floor[i][j][k][l] >= 0{
						_features_features_0_features_0_0_Conv_output_0_floor[i][j][k][l] = BN254Fr::from((i_input._features_features_0_features_0_0_Conv_output_0_floor[i][j][k][l]) as u64); 
					} else {
						_features_features_0_features_0_0_Conv_output_0_floor[i][j][k][l] = -BN254Fr::from((-i_input._features_features_0_features_0_0_Conv_output_0_floor[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_0_features_0_0_Conv_output_0_relu = vec![vec![vec![vec![BN254Fr::default();112];112];32];4]; 
	for i in 0..4 {
		for j in 0..32 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_0_features_0_0_Conv_output_0_relu[i][j][k][l] >= 0{
						_features_features_0_features_0_0_Conv_output_0_relu[i][j][k][l] = BN254Fr::from((i_input._features_features_0_features_0_0_Conv_output_0_relu[i][j][k][l]) as u64); 
					} else {
						_features_features_0_features_0_0_Conv_output_0_relu[i][j][k][l] = -BN254Fr::from((-i_input._features_features_0_features_0_0_Conv_output_0_relu[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_0_features_0_0_Conv_output_0_min = vec![vec![vec![vec![BN254Fr::default();112];112];32];4]; 
	for i in 0..4 {
		for j in 0..32 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_0_features_0_0_Conv_output_0_min[i][j][k][l] >= 0{
						_features_features_0_features_0_0_Conv_output_0_min[i][j][k][l] = BN254Fr::from((i_input._features_features_0_features_0_0_Conv_output_0_min[i][j][k][l]) as u64); 
					} else {
						_features_features_0_features_0_0_Conv_output_0_min[i][j][k][l] = -BN254Fr::from((-i_input._features_features_0_features_0_0_Conv_output_0_min[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_0_features_0_2_PRelu_output_0 = vec![vec![vec![vec![BN254Fr::default();112];112];32];4]; 
	for i in 0..4 {
		for j in 0..32 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_0_features_0_2_PRelu_output_0[i][j][k][l] >= 0{
						_features_features_0_features_0_2_PRelu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_0_features_0_2_PRelu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_0_features_0_2_PRelu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_0_features_0_2_PRelu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();112];112];32];4]; 
	for i in 0..4 {
		for j in 0..32 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l] >= 0{
						_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l] = BN254Fr::from((i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l]) as u64); 
					} else {
						_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l] = -BN254Fr::from((-i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();112];112];32];4]; 
	for i in 0..4 {
		for j in 0..32 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l] >= 0{
						_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l] = BN254Fr::from((i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l]) as u64); 
					} else {
						_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l] = -BN254Fr::from((-i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu = vec![vec![vec![vec![BN254Fr::default();112];112];32];4]; 
	for i in 0..4 {
		for j in 0..32 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l] >= 0{
						_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l] = BN254Fr::from((i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l]) as u64); 
					} else {
						_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l] = -BN254Fr::from((-i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min = vec![vec![vec![vec![BN254Fr::default();112];112];32];4]; 
	for i in 0..4 {
		for j in 0..32 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l] >= 0{
						_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l] = BN254Fr::from((i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l]) as u64); 
					} else {
						_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l] = -BN254Fr::from((-i_input._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_1_conv_conv_0_conv_0_2_PRelu_output_0 = vec![vec![vec![vec![BN254Fr::default();112];112];32];4]; 
	for i in 0..4 {
		for j in 0..32 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_1_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l] >= 0{
						_features_features_1_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_1_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_1_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_1_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_1_conv_conv_1_Conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();112];112];16];4]; 
	for i in 0..4 {
		for j in 0..16 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_1_conv_conv_1_Conv_output_0_conv[i][j][k][l] >= 0{
						_features_features_1_conv_conv_1_Conv_output_0_conv[i][j][k][l] = BN254Fr::from((i_input._features_features_1_conv_conv_1_Conv_output_0_conv[i][j][k][l]) as u64); 
					} else {
						_features_features_1_conv_conv_1_Conv_output_0_conv[i][j][k][l] = -BN254Fr::from((-i_input._features_features_1_conv_conv_1_Conv_output_0_conv[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_1_conv_conv_1_Conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();112];112];16];4]; 
	for i in 0..4 {
		for j in 0..16 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_1_conv_conv_1_Conv_output_0_floor[i][j][k][l] >= 0{
						_features_features_1_conv_conv_1_Conv_output_0_floor[i][j][k][l] = BN254Fr::from((i_input._features_features_1_conv_conv_1_Conv_output_0_floor[i][j][k][l]) as u64); 
					} else {
						_features_features_1_conv_conv_1_Conv_output_0_floor[i][j][k][l] = -BN254Fr::from((-i_input._features_features_1_conv_conv_1_Conv_output_0_floor[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();112];112];96];4]; 
	for i in 0..4 {
		for j in 0..96 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l] >= 0{
						_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();112];112];96];4]; 
	for i in 0..4 {
		for j in 0..96 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l] >= 0{
						_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu = vec![vec![vec![vec![BN254Fr::default();112];112];96];4]; 
	for i in 0..4 {
		for j in 0..96 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l] >= 0{
						_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min = vec![vec![vec![vec![BN254Fr::default();112];112];96];4]; 
	for i in 0..4 {
		for j in 0..96 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l] >= 0{
						_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_0_conv_0_2_PRelu_output_0 = vec![vec![vec![vec![BN254Fr::default();112];112];96];4]; 
	for i in 0..4 {
		for j in 0..96 {
			for k in 0..112 {
				for l in 0..112 {
					if i_input._features_features_2_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l] >= 0{
						_features_features_2_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_1_conv_1_0_Conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();56];56];96];4]; 
	for i in 0..4 {
		for j in 0..96 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_conv[i][j][k][l] >= 0{
						_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_conv[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_conv[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_conv[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_conv[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_1_conv_1_0_Conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();56];56];96];4]; 
	for i in 0..4 {
		for j in 0..96 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_floor[i][j][k][l] >= 0{
						_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_floor[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_floor[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_floor[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_floor[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_1_conv_1_0_Conv_output_0_relu = vec![vec![vec![vec![BN254Fr::default();56];56];96];4]; 
	for i in 0..4 {
		for j in 0..96 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_relu[i][j][k][l] >= 0{
						_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_relu[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_relu[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_relu[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_relu[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_1_conv_1_0_Conv_output_0_min = vec![vec![vec![vec![BN254Fr::default();56];56];96];4]; 
	for i in 0..4 {
		for j in 0..96 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_min[i][j][k][l] >= 0{
						_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_min[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_min[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_min[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_1_conv_1_0_Conv_output_0_min[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_1_conv_1_2_PRelu_output_0 = vec![vec![vec![vec![BN254Fr::default();56];56];96];4]; 
	for i in 0..4 {
		for j in 0..96 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_2_conv_conv_1_conv_1_2_PRelu_output_0[i][j][k][l] >= 0{
						_features_features_2_conv_conv_1_conv_1_2_PRelu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_1_conv_1_2_PRelu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_1_conv_1_2_PRelu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_1_conv_1_2_PRelu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_2_Conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();56];56];24];4]; 
	for i in 0..4 {
		for j in 0..24 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_2_conv_conv_2_Conv_output_0_conv[i][j][k][l] >= 0{
						_features_features_2_conv_conv_2_Conv_output_0_conv[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_2_Conv_output_0_conv[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_2_Conv_output_0_conv[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_2_Conv_output_0_conv[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_2_conv_conv_2_Conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();56];56];24];4]; 
	for i in 0..4 {
		for j in 0..24 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_2_conv_conv_2_Conv_output_0_floor[i][j][k][l] >= 0{
						_features_features_2_conv_conv_2_Conv_output_0_floor[i][j][k][l] = BN254Fr::from((i_input._features_features_2_conv_conv_2_Conv_output_0_floor[i][j][k][l]) as u64); 
					} else {
						_features_features_2_conv_conv_2_Conv_output_0_floor[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_conv_conv_2_Conv_output_0_floor[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_3_conv_conv_0_conv_0_0_Conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();56];56];144];4]; 
	for i in 0..4 {
		for j in 0..144 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l] >= 0{
						_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l] = BN254Fr::from((i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l]) as u64); 
					} else {
						_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l] = -BN254Fr::from((-i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_3_conv_conv_0_conv_0_0_Conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();56];56];144];4]; 
	for i in 0..4 {
		for j in 0..144 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l] >= 0{
						_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l] = BN254Fr::from((i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l]) as u64); 
					} else {
						_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l] = -BN254Fr::from((-i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_3_conv_conv_0_conv_0_0_Conv_output_0_relu = vec![vec![vec![vec![BN254Fr::default();56];56];144];4]; 
	for i in 0..4 {
		for j in 0..144 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l] >= 0{
						_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l] = BN254Fr::from((i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l]) as u64); 
					} else {
						_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l] = -BN254Fr::from((-i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_3_conv_conv_0_conv_0_0_Conv_output_0_min = vec![vec![vec![vec![BN254Fr::default();56];56];144];4]; 
	for i in 0..4 {
		for j in 0..144 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l] >= 0{
						_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l] = BN254Fr::from((i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l]) as u64); 
					} else {
						_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l] = -BN254Fr::from((-i_input._features_features_3_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_3_conv_conv_0_conv_0_2_PRelu_output_0 = vec![vec![vec![vec![BN254Fr::default();56];56];144];4]; 
	for i in 0..4 {
		for j in 0..144 {
			for k in 0..56 {
				for l in 0..56 {
					if i_input._features_features_3_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l] >= 0{
						_features_features_3_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_3_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_3_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_3_conv_conv_0_conv_0_2_PRelu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_621 = vec![vec![vec![vec![BN254Fr::default();3];3];3];32]; 
	for i in 0..32 {
		for j in 0..3 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_621[i][j][k][l] >= 0{
						onnx__Conv_621[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_621[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_621[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_621[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_622 = vec![BN254Fr::default();32]; 
	for i in 0..32 {
		if i_input.onnx__Conv_622[i] >= 0{
			onnx__Conv_622[i] = BN254Fr::from((i_input.onnx__Conv_622[i]) as u64); 
		} else {
			onnx__Conv_622[i] = -BN254Fr::from((-i_input.onnx__Conv_622[i]) as u64); 
		} 
	}
	let mut onnx__Conv_622_q = vec![vec![vec![BN254Fr::default();1];1];32]; 
	for i in 0..32 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_622_q[i][j][k] >= 0{
					onnx__Conv_622_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_622_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_622_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_622_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_621_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_621_nscale >= 0{
		onnx__Conv_621_nscale = BN254Fr::from((i_input.onnx__Conv_621_nscale) as u64); 
	} else {
		onnx__Conv_621_nscale = -BN254Fr::from((-i_input.onnx__Conv_621_nscale) as u64); 
	} 
	let mut onnx__Conv_621_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_621_dscale >= 0{
		onnx__Conv_621_dscale = BN254Fr::from((i_input.onnx__Conv_621_dscale) as u64); 
	} else {
		onnx__Conv_621_dscale = -BN254Fr::from((-i_input.onnx__Conv_621_dscale) as u64); 
	} 
	let mut onnx__PRelu_779_q = vec![vec![vec![BN254Fr::default();1];1];32]; 
	for i in 0..32 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_779_q[i][j][k] >= 0{
					onnx__PRelu_779_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_779_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_779_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_779_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_779_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_779_nscale >= 0{
		onnx__PRelu_779_nscale = BN254Fr::from((i_input.onnx__PRelu_779_nscale) as u64); 
	} else {
		onnx__PRelu_779_nscale = -BN254Fr::from((-i_input.onnx__PRelu_779_nscale) as u64); 
	} 
	let mut onnx__PRelu_779_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_779_dscale >= 0{
		onnx__PRelu_779_dscale = BN254Fr::from((i_input.onnx__PRelu_779_dscale) as u64); 
	} else {
		onnx__PRelu_779_dscale = -BN254Fr::from((-i_input.onnx__PRelu_779_dscale) as u64); 
	} 
	let mut onnx__PRelu_779_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_779_zero[i] >= 0{
			onnx__PRelu_779_zero[i] = BN254Fr::from((i_input.onnx__PRelu_779_zero[i]) as u64); 
		} else {
			onnx__PRelu_779_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_779_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_624 = vec![vec![vec![vec![BN254Fr::default();3];3];1];32]; 
	for i in 0..32 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_624[i][j][k][l] >= 0{
						onnx__Conv_624[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_624[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_624[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_624[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_625 = vec![BN254Fr::default();32]; 
	for i in 0..32 {
		if i_input.onnx__Conv_625[i] >= 0{
			onnx__Conv_625[i] = BN254Fr::from((i_input.onnx__Conv_625[i]) as u64); 
		} else {
			onnx__Conv_625[i] = -BN254Fr::from((-i_input.onnx__Conv_625[i]) as u64); 
		} 
	}
	let mut onnx__Conv_625_q = vec![vec![vec![BN254Fr::default();1];1];32]; 
	for i in 0..32 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_625_q[i][j][k] >= 0{
					onnx__Conv_625_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_625_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_625_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_625_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_624_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_624_nscale >= 0{
		onnx__Conv_624_nscale = BN254Fr::from((i_input.onnx__Conv_624_nscale) as u64); 
	} else {
		onnx__Conv_624_nscale = -BN254Fr::from((-i_input.onnx__Conv_624_nscale) as u64); 
	} 
	let mut onnx__Conv_624_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_624_dscale >= 0{
		onnx__Conv_624_dscale = BN254Fr::from((i_input.onnx__Conv_624_dscale) as u64); 
	} else {
		onnx__Conv_624_dscale = -BN254Fr::from((-i_input.onnx__Conv_624_dscale) as u64); 
	} 
	let mut onnx__PRelu_780_q = vec![vec![vec![BN254Fr::default();1];1];32]; 
	for i in 0..32 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_780_q[i][j][k] >= 0{
					onnx__PRelu_780_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_780_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_780_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_780_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_780_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_780_nscale >= 0{
		onnx__PRelu_780_nscale = BN254Fr::from((i_input.onnx__PRelu_780_nscale) as u64); 
	} else {
		onnx__PRelu_780_nscale = -BN254Fr::from((-i_input.onnx__PRelu_780_nscale) as u64); 
	} 
	let mut onnx__PRelu_780_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_780_dscale >= 0{
		onnx__PRelu_780_dscale = BN254Fr::from((i_input.onnx__PRelu_780_dscale) as u64); 
	} else {
		onnx__PRelu_780_dscale = -BN254Fr::from((-i_input.onnx__PRelu_780_dscale) as u64); 
	} 
	let mut onnx__PRelu_780_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_780_zero[i] >= 0{
			onnx__PRelu_780_zero[i] = BN254Fr::from((i_input.onnx__PRelu_780_zero[i]) as u64); 
		} else {
			onnx__PRelu_780_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_780_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_627 = vec![vec![vec![vec![BN254Fr::default();1];1];32];16]; 
	for i in 0..16 {
		for j in 0..32 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_627[i][j][k][l] >= 0{
						onnx__Conv_627[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_627[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_627[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_627[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_628 = vec![BN254Fr::default();16]; 
	for i in 0..16 {
		if i_input.onnx__Conv_628[i] >= 0{
			onnx__Conv_628[i] = BN254Fr::from((i_input.onnx__Conv_628[i]) as u64); 
		} else {
			onnx__Conv_628[i] = -BN254Fr::from((-i_input.onnx__Conv_628[i]) as u64); 
		} 
	}
	let mut onnx__Conv_628_q = vec![vec![vec![BN254Fr::default();1];1];16]; 
	for i in 0..16 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_628_q[i][j][k] >= 0{
					onnx__Conv_628_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_628_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_628_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_628_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_627_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_627_nscale >= 0{
		onnx__Conv_627_nscale = BN254Fr::from((i_input.onnx__Conv_627_nscale) as u64); 
	} else {
		onnx__Conv_627_nscale = -BN254Fr::from((-i_input.onnx__Conv_627_nscale) as u64); 
	} 
	let mut onnx__Conv_627_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_627_dscale >= 0{
		onnx__Conv_627_dscale = BN254Fr::from((i_input.onnx__Conv_627_dscale) as u64); 
	} else {
		onnx__Conv_627_dscale = -BN254Fr::from((-i_input.onnx__Conv_627_dscale) as u64); 
	} 
	let mut onnx__Conv_630 = vec![vec![vec![vec![BN254Fr::default();1];1];16];96]; 
	for i in 0..96 {
		for j in 0..16 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_630[i][j][k][l] >= 0{
						onnx__Conv_630[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_630[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_630[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_630[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_631 = vec![BN254Fr::default();96]; 
	for i in 0..96 {
		if i_input.onnx__Conv_631[i] >= 0{
			onnx__Conv_631[i] = BN254Fr::from((i_input.onnx__Conv_631[i]) as u64); 
		} else {
			onnx__Conv_631[i] = -BN254Fr::from((-i_input.onnx__Conv_631[i]) as u64); 
		} 
	}
	let mut onnx__Conv_631_q = vec![vec![vec![BN254Fr::default();1];1];96]; 
	for i in 0..96 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_631_q[i][j][k] >= 0{
					onnx__Conv_631_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_631_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_631_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_631_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_630_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_630_nscale >= 0{
		onnx__Conv_630_nscale = BN254Fr::from((i_input.onnx__Conv_630_nscale) as u64); 
	} else {
		onnx__Conv_630_nscale = -BN254Fr::from((-i_input.onnx__Conv_630_nscale) as u64); 
	} 
	let mut onnx__Conv_630_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_630_dscale >= 0{
		onnx__Conv_630_dscale = BN254Fr::from((i_input.onnx__Conv_630_dscale) as u64); 
	} else {
		onnx__Conv_630_dscale = -BN254Fr::from((-i_input.onnx__Conv_630_dscale) as u64); 
	} 
	let mut onnx__PRelu_781_q = vec![vec![vec![BN254Fr::default();1];1];96]; 
	for i in 0..96 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_781_q[i][j][k] >= 0{
					onnx__PRelu_781_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_781_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_781_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_781_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_781_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_781_nscale >= 0{
		onnx__PRelu_781_nscale = BN254Fr::from((i_input.onnx__PRelu_781_nscale) as u64); 
	} else {
		onnx__PRelu_781_nscale = -BN254Fr::from((-i_input.onnx__PRelu_781_nscale) as u64); 
	} 
	let mut onnx__PRelu_781_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_781_dscale >= 0{
		onnx__PRelu_781_dscale = BN254Fr::from((i_input.onnx__PRelu_781_dscale) as u64); 
	} else {
		onnx__PRelu_781_dscale = -BN254Fr::from((-i_input.onnx__PRelu_781_dscale) as u64); 
	} 
	let mut onnx__PRelu_781_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_781_zero[i] >= 0{
			onnx__PRelu_781_zero[i] = BN254Fr::from((i_input.onnx__PRelu_781_zero[i]) as u64); 
		} else {
			onnx__PRelu_781_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_781_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_633 = vec![vec![vec![vec![BN254Fr::default();3];3];1];96]; 
	for i in 0..96 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_633[i][j][k][l] >= 0{
						onnx__Conv_633[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_633[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_633[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_633[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_634 = vec![BN254Fr::default();96]; 
	for i in 0..96 {
		if i_input.onnx__Conv_634[i] >= 0{
			onnx__Conv_634[i] = BN254Fr::from((i_input.onnx__Conv_634[i]) as u64); 
		} else {
			onnx__Conv_634[i] = -BN254Fr::from((-i_input.onnx__Conv_634[i]) as u64); 
		} 
	}
	let mut onnx__Conv_634_q = vec![vec![vec![BN254Fr::default();1];1];96]; 
	for i in 0..96 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_634_q[i][j][k] >= 0{
					onnx__Conv_634_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_634_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_634_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_634_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_633_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_633_nscale >= 0{
		onnx__Conv_633_nscale = BN254Fr::from((i_input.onnx__Conv_633_nscale) as u64); 
	} else {
		onnx__Conv_633_nscale = -BN254Fr::from((-i_input.onnx__Conv_633_nscale) as u64); 
	} 
	let mut onnx__Conv_633_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_633_dscale >= 0{
		onnx__Conv_633_dscale = BN254Fr::from((i_input.onnx__Conv_633_dscale) as u64); 
	} else {
		onnx__Conv_633_dscale = -BN254Fr::from((-i_input.onnx__Conv_633_dscale) as u64); 
	} 
	let mut onnx__PRelu_782_q = vec![vec![vec![BN254Fr::default();1];1];96]; 
	for i in 0..96 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_782_q[i][j][k] >= 0{
					onnx__PRelu_782_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_782_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_782_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_782_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_782_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_782_nscale >= 0{
		onnx__PRelu_782_nscale = BN254Fr::from((i_input.onnx__PRelu_782_nscale) as u64); 
	} else {
		onnx__PRelu_782_nscale = -BN254Fr::from((-i_input.onnx__PRelu_782_nscale) as u64); 
	} 
	let mut onnx__PRelu_782_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_782_dscale >= 0{
		onnx__PRelu_782_dscale = BN254Fr::from((i_input.onnx__PRelu_782_dscale) as u64); 
	} else {
		onnx__PRelu_782_dscale = -BN254Fr::from((-i_input.onnx__PRelu_782_dscale) as u64); 
	} 
	let mut onnx__PRelu_782_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_782_zero[i] >= 0{
			onnx__PRelu_782_zero[i] = BN254Fr::from((i_input.onnx__PRelu_782_zero[i]) as u64); 
		} else {
			onnx__PRelu_782_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_782_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_636 = vec![vec![vec![vec![BN254Fr::default();1];1];96];24]; 
	for i in 0..24 {
		for j in 0..96 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_636[i][j][k][l] >= 0{
						onnx__Conv_636[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_636[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_636[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_636[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_637 = vec![BN254Fr::default();24]; 
	for i in 0..24 {
		if i_input.onnx__Conv_637[i] >= 0{
			onnx__Conv_637[i] = BN254Fr::from((i_input.onnx__Conv_637[i]) as u64); 
		} else {
			onnx__Conv_637[i] = -BN254Fr::from((-i_input.onnx__Conv_637[i]) as u64); 
		} 
	}
	let mut onnx__Conv_637_q = vec![vec![vec![BN254Fr::default();1];1];24]; 
	for i in 0..24 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_637_q[i][j][k] >= 0{
					onnx__Conv_637_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_637_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_637_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_637_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_636_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_636_nscale >= 0{
		onnx__Conv_636_nscale = BN254Fr::from((i_input.onnx__Conv_636_nscale) as u64); 
	} else {
		onnx__Conv_636_nscale = -BN254Fr::from((-i_input.onnx__Conv_636_nscale) as u64); 
	} 
	let mut onnx__Conv_636_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_636_dscale >= 0{
		onnx__Conv_636_dscale = BN254Fr::from((i_input.onnx__Conv_636_dscale) as u64); 
	} else {
		onnx__Conv_636_dscale = -BN254Fr::from((-i_input.onnx__Conv_636_dscale) as u64); 
	} 
	let mut onnx__Conv_639 = vec![vec![vec![vec![BN254Fr::default();1];1];24];144]; 
	for i in 0..144 {
		for j in 0..24 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_639[i][j][k][l] >= 0{
						onnx__Conv_639[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_639[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_639[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_639[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_640 = vec![BN254Fr::default();144]; 
	for i in 0..144 {
		if i_input.onnx__Conv_640[i] >= 0{
			onnx__Conv_640[i] = BN254Fr::from((i_input.onnx__Conv_640[i]) as u64); 
		} else {
			onnx__Conv_640[i] = -BN254Fr::from((-i_input.onnx__Conv_640[i]) as u64); 
		} 
	}
	let mut onnx__Conv_640_q = vec![vec![vec![BN254Fr::default();1];1];144]; 
	for i in 0..144 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_640_q[i][j][k] >= 0{
					onnx__Conv_640_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_640_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_640_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_640_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_639_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_639_nscale >= 0{
		onnx__Conv_639_nscale = BN254Fr::from((i_input.onnx__Conv_639_nscale) as u64); 
	} else {
		onnx__Conv_639_nscale = -BN254Fr::from((-i_input.onnx__Conv_639_nscale) as u64); 
	} 
	let mut onnx__Conv_639_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_639_dscale >= 0{
		onnx__Conv_639_dscale = BN254Fr::from((i_input.onnx__Conv_639_dscale) as u64); 
	} else {
		onnx__Conv_639_dscale = -BN254Fr::from((-i_input.onnx__Conv_639_dscale) as u64); 
	} 
	let mut onnx__PRelu_783_q = vec![vec![vec![BN254Fr::default();1];1];144]; 
	for i in 0..144 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_783_q[i][j][k] >= 0{
					onnx__PRelu_783_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_783_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_783_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_783_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_783_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_783_nscale >= 0{
		onnx__PRelu_783_nscale = BN254Fr::from((i_input.onnx__PRelu_783_nscale) as u64); 
	} else {
		onnx__PRelu_783_nscale = -BN254Fr::from((-i_input.onnx__PRelu_783_nscale) as u64); 
	} 
	let mut onnx__PRelu_783_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_783_dscale >= 0{
		onnx__PRelu_783_dscale = BN254Fr::from((i_input.onnx__PRelu_783_dscale) as u64); 
	} else {
		onnx__PRelu_783_dscale = -BN254Fr::from((-i_input.onnx__PRelu_783_dscale) as u64); 
	} 
	let mut onnx__PRelu_783_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_783_zero[i] >= 0{
			onnx__PRelu_783_zero[i] = BN254Fr::from((i_input.onnx__PRelu_783_zero[i]) as u64); 
		} else {
			onnx__PRelu_783_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_783_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_642 = vec![vec![vec![vec![BN254Fr::default();3];3];1];144]; 
	for i in 0..144 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_642[i][j][k][l] >= 0{
						onnx__Conv_642[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_642[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_642[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_642[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_643 = vec![BN254Fr::default();144]; 
	for i in 0..144 {
		if i_input.onnx__Conv_643[i] >= 0{
			onnx__Conv_643[i] = BN254Fr::from((i_input.onnx__Conv_643[i]) as u64); 
		} else {
			onnx__Conv_643[i] = -BN254Fr::from((-i_input.onnx__Conv_643[i]) as u64); 
		} 
	}
	let mut onnx__Conv_643_q = vec![vec![vec![BN254Fr::default();1];1];144]; 
	for i in 0..144 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_643_q[i][j][k] >= 0{
					onnx__Conv_643_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_643_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_643_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_643_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_642_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_642_nscale >= 0{
		onnx__Conv_642_nscale = BN254Fr::from((i_input.onnx__Conv_642_nscale) as u64); 
	} else {
		onnx__Conv_642_nscale = -BN254Fr::from((-i_input.onnx__Conv_642_nscale) as u64); 
	} 
	let mut onnx__Conv_642_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_642_dscale >= 0{
		onnx__Conv_642_dscale = BN254Fr::from((i_input.onnx__Conv_642_dscale) as u64); 
	} else {
		onnx__Conv_642_dscale = -BN254Fr::from((-i_input.onnx__Conv_642_dscale) as u64); 
	} 
	let mut onnx__PRelu_784_q = vec![vec![vec![BN254Fr::default();1];1];144]; 
	for i in 0..144 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_784_q[i][j][k] >= 0{
					onnx__PRelu_784_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_784_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_784_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_784_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_784_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_784_nscale >= 0{
		onnx__PRelu_784_nscale = BN254Fr::from((i_input.onnx__PRelu_784_nscale) as u64); 
	} else {
		onnx__PRelu_784_nscale = -BN254Fr::from((-i_input.onnx__PRelu_784_nscale) as u64); 
	} 
	let mut onnx__PRelu_784_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_784_dscale >= 0{
		onnx__PRelu_784_dscale = BN254Fr::from((i_input.onnx__PRelu_784_dscale) as u64); 
	} else {
		onnx__PRelu_784_dscale = -BN254Fr::from((-i_input.onnx__PRelu_784_dscale) as u64); 
	} 
	let mut onnx__PRelu_784_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_784_zero[i] >= 0{
			onnx__PRelu_784_zero[i] = BN254Fr::from((i_input.onnx__PRelu_784_zero[i]) as u64); 
		} else {
			onnx__PRelu_784_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_784_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_645 = vec![vec![vec![vec![BN254Fr::default();1];1];144];24]; 
	for i in 0..24 {
		for j in 0..144 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_645[i][j][k][l] >= 0{
						onnx__Conv_645[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_645[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_645[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_645[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_646 = vec![BN254Fr::default();24]; 
	for i in 0..24 {
		if i_input.onnx__Conv_646[i] >= 0{
			onnx__Conv_646[i] = BN254Fr::from((i_input.onnx__Conv_646[i]) as u64); 
		} else {
			onnx__Conv_646[i] = -BN254Fr::from((-i_input.onnx__Conv_646[i]) as u64); 
		} 
	}
	let mut onnx__Conv_646_q = vec![vec![vec![BN254Fr::default();1];1];24]; 
	for i in 0..24 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_646_q[i][j][k] >= 0{
					onnx__Conv_646_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_646_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_646_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_646_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_645_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_645_nscale >= 0{
		onnx__Conv_645_nscale = BN254Fr::from((i_input.onnx__Conv_645_nscale) as u64); 
	} else {
		onnx__Conv_645_nscale = -BN254Fr::from((-i_input.onnx__Conv_645_nscale) as u64); 
	} 
	let mut onnx__Conv_645_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_645_dscale >= 0{
		onnx__Conv_645_dscale = BN254Fr::from((i_input.onnx__Conv_645_dscale) as u64); 
	} else {
		onnx__Conv_645_dscale = -BN254Fr::from((-i_input.onnx__Conv_645_dscale) as u64); 
	} 
	let mut _features_features_3_Add_output_0_1nscale = BN254Fr::default(); 
	if i_input._features_features_3_Add_output_0_1nscale >= 0{
		_features_features_3_Add_output_0_1nscale = BN254Fr::from((i_input._features_features_3_Add_output_0_1nscale) as u64); 
	} else {
		_features_features_3_Add_output_0_1nscale = -BN254Fr::from((-i_input._features_features_3_Add_output_0_1nscale) as u64); 
	} 
	let mut _features_features_3_Add_output_0_1dscale = BN254Fr::default(); 
	if i_input._features_features_3_Add_output_0_1dscale >= 0{
		_features_features_3_Add_output_0_1dscale = BN254Fr::from((i_input._features_features_3_Add_output_0_1dscale) as u64); 
	} else {
		_features_features_3_Add_output_0_1dscale = -BN254Fr::from((-i_input._features_features_3_Add_output_0_1dscale) as u64); 
	} 
	let mut _features_features_3_Add_output_0_2nscale = BN254Fr::default(); 
	if i_input._features_features_3_Add_output_0_2nscale >= 0{
		_features_features_3_Add_output_0_2nscale = BN254Fr::from((i_input._features_features_3_Add_output_0_2nscale) as u64); 
	} else {
		_features_features_3_Add_output_0_2nscale = -BN254Fr::from((-i_input._features_features_3_Add_output_0_2nscale) as u64); 
	} 
	let mut _features_features_3_Add_output_0_2dscale = BN254Fr::default(); 
	if i_input._features_features_3_Add_output_0_2dscale >= 0{
		_features_features_3_Add_output_0_2dscale = BN254Fr::from((i_input._features_features_3_Add_output_0_2dscale) as u64); 
	} else {
		_features_features_3_Add_output_0_2dscale = -BN254Fr::from((-i_input._features_features_3_Add_output_0_2dscale) as u64); 
	} 
	let mut onnx__Conv_648 = vec![vec![vec![vec![BN254Fr::default();1];1];24];144]; 
	for i in 0..144 {
		for j in 0..24 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_648[i][j][k][l] >= 0{
						onnx__Conv_648[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_648[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_648[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_648[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_649 = vec![BN254Fr::default();144]; 
	for i in 0..144 {
		if i_input.onnx__Conv_649[i] >= 0{
			onnx__Conv_649[i] = BN254Fr::from((i_input.onnx__Conv_649[i]) as u64); 
		} else {
			onnx__Conv_649[i] = -BN254Fr::from((-i_input.onnx__Conv_649[i]) as u64); 
		} 
	}
	let mut onnx__Conv_649_q = vec![vec![vec![BN254Fr::default();1];1];144]; 
	for i in 0..144 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_649_q[i][j][k] >= 0{
					onnx__Conv_649_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_649_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_649_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_649_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_648_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_648_nscale >= 0{
		onnx__Conv_648_nscale = BN254Fr::from((i_input.onnx__Conv_648_nscale) as u64); 
	} else {
		onnx__Conv_648_nscale = -BN254Fr::from((-i_input.onnx__Conv_648_nscale) as u64); 
	} 
	let mut onnx__Conv_648_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_648_dscale >= 0{
		onnx__Conv_648_dscale = BN254Fr::from((i_input.onnx__Conv_648_dscale) as u64); 
	} else {
		onnx__Conv_648_dscale = -BN254Fr::from((-i_input.onnx__Conv_648_dscale) as u64); 
	} 
	let mut onnx__PRelu_785_q = vec![vec![vec![BN254Fr::default();1];1];144]; 
	for i in 0..144 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_785_q[i][j][k] >= 0{
					onnx__PRelu_785_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_785_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_785_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_785_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_785_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_785_nscale >= 0{
		onnx__PRelu_785_nscale = BN254Fr::from((i_input.onnx__PRelu_785_nscale) as u64); 
	} else {
		onnx__PRelu_785_nscale = -BN254Fr::from((-i_input.onnx__PRelu_785_nscale) as u64); 
	} 
	let mut onnx__PRelu_785_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_785_dscale >= 0{
		onnx__PRelu_785_dscale = BN254Fr::from((i_input.onnx__PRelu_785_dscale) as u64); 
	} else {
		onnx__PRelu_785_dscale = -BN254Fr::from((-i_input.onnx__PRelu_785_dscale) as u64); 
	} 
	let mut onnx__PRelu_785_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_785_zero[i] >= 0{
			onnx__PRelu_785_zero[i] = BN254Fr::from((i_input.onnx__PRelu_785_zero[i]) as u64); 
		} else {
			onnx__PRelu_785_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_785_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_651 = vec![vec![vec![vec![BN254Fr::default();3];3];1];144]; 
	for i in 0..144 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_651[i][j][k][l] >= 0{
						onnx__Conv_651[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_651[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_651[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_651[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_652 = vec![BN254Fr::default();144]; 
	for i in 0..144 {
		if i_input.onnx__Conv_652[i] >= 0{
			onnx__Conv_652[i] = BN254Fr::from((i_input.onnx__Conv_652[i]) as u64); 
		} else {
			onnx__Conv_652[i] = -BN254Fr::from((-i_input.onnx__Conv_652[i]) as u64); 
		} 
	}
	let mut onnx__Conv_652_q = vec![vec![vec![BN254Fr::default();1];1];144]; 
	for i in 0..144 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_652_q[i][j][k] >= 0{
					onnx__Conv_652_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_652_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_652_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_652_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_651_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_651_nscale >= 0{
		onnx__Conv_651_nscale = BN254Fr::from((i_input.onnx__Conv_651_nscale) as u64); 
	} else {
		onnx__Conv_651_nscale = -BN254Fr::from((-i_input.onnx__Conv_651_nscale) as u64); 
	} 
	let mut onnx__Conv_651_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_651_dscale >= 0{
		onnx__Conv_651_dscale = BN254Fr::from((i_input.onnx__Conv_651_dscale) as u64); 
	} else {
		onnx__Conv_651_dscale = -BN254Fr::from((-i_input.onnx__Conv_651_dscale) as u64); 
	} 
	let mut onnx__PRelu_786_q = vec![vec![vec![BN254Fr::default();1];1];144]; 
	for i in 0..144 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_786_q[i][j][k] >= 0{
					onnx__PRelu_786_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_786_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_786_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_786_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_786_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_786_nscale >= 0{
		onnx__PRelu_786_nscale = BN254Fr::from((i_input.onnx__PRelu_786_nscale) as u64); 
	} else {
		onnx__PRelu_786_nscale = -BN254Fr::from((-i_input.onnx__PRelu_786_nscale) as u64); 
	} 
	let mut onnx__PRelu_786_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_786_dscale >= 0{
		onnx__PRelu_786_dscale = BN254Fr::from((i_input.onnx__PRelu_786_dscale) as u64); 
	} else {
		onnx__PRelu_786_dscale = -BN254Fr::from((-i_input.onnx__PRelu_786_dscale) as u64); 
	} 
	let mut onnx__PRelu_786_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_786_zero[i] >= 0{
			onnx__PRelu_786_zero[i] = BN254Fr::from((i_input.onnx__PRelu_786_zero[i]) as u64); 
		} else {
			onnx__PRelu_786_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_786_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_654 = vec![vec![vec![vec![BN254Fr::default();1];1];144];32]; 
	for i in 0..32 {
		for j in 0..144 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_654[i][j][k][l] >= 0{
						onnx__Conv_654[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_654[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_654[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_654[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_655 = vec![BN254Fr::default();32]; 
	for i in 0..32 {
		if i_input.onnx__Conv_655[i] >= 0{
			onnx__Conv_655[i] = BN254Fr::from((i_input.onnx__Conv_655[i]) as u64); 
		} else {
			onnx__Conv_655[i] = -BN254Fr::from((-i_input.onnx__Conv_655[i]) as u64); 
		} 
	}
	let mut onnx__Conv_655_q = vec![vec![vec![BN254Fr::default();1];1];32]; 
	for i in 0..32 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_655_q[i][j][k] >= 0{
					onnx__Conv_655_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_655_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_655_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_655_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_654_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_654_nscale >= 0{
		onnx__Conv_654_nscale = BN254Fr::from((i_input.onnx__Conv_654_nscale) as u64); 
	} else {
		onnx__Conv_654_nscale = -BN254Fr::from((-i_input.onnx__Conv_654_nscale) as u64); 
	} 
	let mut onnx__Conv_654_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_654_dscale >= 0{
		onnx__Conv_654_dscale = BN254Fr::from((i_input.onnx__Conv_654_dscale) as u64); 
	} else {
		onnx__Conv_654_dscale = -BN254Fr::from((-i_input.onnx__Conv_654_dscale) as u64); 
	} 
	let mut onnx__Conv_657 = vec![vec![vec![vec![BN254Fr::default();1];1];32];192]; 
	for i in 0..192 {
		for j in 0..32 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_657[i][j][k][l] >= 0{
						onnx__Conv_657[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_657[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_657[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_657[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_658 = vec![BN254Fr::default();192]; 
	for i in 0..192 {
		if i_input.onnx__Conv_658[i] >= 0{
			onnx__Conv_658[i] = BN254Fr::from((i_input.onnx__Conv_658[i]) as u64); 
		} else {
			onnx__Conv_658[i] = -BN254Fr::from((-i_input.onnx__Conv_658[i]) as u64); 
		} 
	}
	let mut onnx__Conv_658_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_658_q[i][j][k] >= 0{
					onnx__Conv_658_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_658_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_658_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_658_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_657_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_657_nscale >= 0{
		onnx__Conv_657_nscale = BN254Fr::from((i_input.onnx__Conv_657_nscale) as u64); 
	} else {
		onnx__Conv_657_nscale = -BN254Fr::from((-i_input.onnx__Conv_657_nscale) as u64); 
	} 
	let mut onnx__Conv_657_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_657_dscale >= 0{
		onnx__Conv_657_dscale = BN254Fr::from((i_input.onnx__Conv_657_dscale) as u64); 
	} else {
		onnx__Conv_657_dscale = -BN254Fr::from((-i_input.onnx__Conv_657_dscale) as u64); 
	} 
	let mut onnx__PRelu_787_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_787_q[i][j][k] >= 0{
					onnx__PRelu_787_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_787_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_787_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_787_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_787_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_787_nscale >= 0{
		onnx__PRelu_787_nscale = BN254Fr::from((i_input.onnx__PRelu_787_nscale) as u64); 
	} else {
		onnx__PRelu_787_nscale = -BN254Fr::from((-i_input.onnx__PRelu_787_nscale) as u64); 
	} 
	let mut onnx__PRelu_787_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_787_dscale >= 0{
		onnx__PRelu_787_dscale = BN254Fr::from((i_input.onnx__PRelu_787_dscale) as u64); 
	} else {
		onnx__PRelu_787_dscale = -BN254Fr::from((-i_input.onnx__PRelu_787_dscale) as u64); 
	} 
	let mut onnx__PRelu_787_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_787_zero[i] >= 0{
			onnx__PRelu_787_zero[i] = BN254Fr::from((i_input.onnx__PRelu_787_zero[i]) as u64); 
		} else {
			onnx__PRelu_787_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_787_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_660 = vec![vec![vec![vec![BN254Fr::default();3];3];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_660[i][j][k][l] >= 0{
						onnx__Conv_660[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_660[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_660[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_660[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_661 = vec![BN254Fr::default();192]; 
	for i in 0..192 {
		if i_input.onnx__Conv_661[i] >= 0{
			onnx__Conv_661[i] = BN254Fr::from((i_input.onnx__Conv_661[i]) as u64); 
		} else {
			onnx__Conv_661[i] = -BN254Fr::from((-i_input.onnx__Conv_661[i]) as u64); 
		} 
	}
	let mut onnx__Conv_661_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_661_q[i][j][k] >= 0{
					onnx__Conv_661_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_661_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_661_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_661_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_660_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_660_nscale >= 0{
		onnx__Conv_660_nscale = BN254Fr::from((i_input.onnx__Conv_660_nscale) as u64); 
	} else {
		onnx__Conv_660_nscale = -BN254Fr::from((-i_input.onnx__Conv_660_nscale) as u64); 
	} 
	let mut onnx__Conv_660_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_660_dscale >= 0{
		onnx__Conv_660_dscale = BN254Fr::from((i_input.onnx__Conv_660_dscale) as u64); 
	} else {
		onnx__Conv_660_dscale = -BN254Fr::from((-i_input.onnx__Conv_660_dscale) as u64); 
	} 
	let mut onnx__PRelu_788_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_788_q[i][j][k] >= 0{
					onnx__PRelu_788_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_788_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_788_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_788_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_788_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_788_nscale >= 0{
		onnx__PRelu_788_nscale = BN254Fr::from((i_input.onnx__PRelu_788_nscale) as u64); 
	} else {
		onnx__PRelu_788_nscale = -BN254Fr::from((-i_input.onnx__PRelu_788_nscale) as u64); 
	} 
	let mut onnx__PRelu_788_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_788_dscale >= 0{
		onnx__PRelu_788_dscale = BN254Fr::from((i_input.onnx__PRelu_788_dscale) as u64); 
	} else {
		onnx__PRelu_788_dscale = -BN254Fr::from((-i_input.onnx__PRelu_788_dscale) as u64); 
	} 
	let mut onnx__PRelu_788_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_788_zero[i] >= 0{
			onnx__PRelu_788_zero[i] = BN254Fr::from((i_input.onnx__PRelu_788_zero[i]) as u64); 
		} else {
			onnx__PRelu_788_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_788_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_663 = vec![vec![vec![vec![BN254Fr::default();1];1];192];32]; 
	for i in 0..32 {
		for j in 0..192 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_663[i][j][k][l] >= 0{
						onnx__Conv_663[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_663[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_663[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_663[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_664 = vec![BN254Fr::default();32]; 
	for i in 0..32 {
		if i_input.onnx__Conv_664[i] >= 0{
			onnx__Conv_664[i] = BN254Fr::from((i_input.onnx__Conv_664[i]) as u64); 
		} else {
			onnx__Conv_664[i] = -BN254Fr::from((-i_input.onnx__Conv_664[i]) as u64); 
		} 
	}
	let mut onnx__Conv_664_q = vec![vec![vec![BN254Fr::default();1];1];32]; 
	for i in 0..32 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_664_q[i][j][k] >= 0{
					onnx__Conv_664_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_664_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_664_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_664_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_663_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_663_nscale >= 0{
		onnx__Conv_663_nscale = BN254Fr::from((i_input.onnx__Conv_663_nscale) as u64); 
	} else {
		onnx__Conv_663_nscale = -BN254Fr::from((-i_input.onnx__Conv_663_nscale) as u64); 
	} 
	let mut onnx__Conv_663_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_663_dscale >= 0{
		onnx__Conv_663_dscale = BN254Fr::from((i_input.onnx__Conv_663_dscale) as u64); 
	} else {
		onnx__Conv_663_dscale = -BN254Fr::from((-i_input.onnx__Conv_663_dscale) as u64); 
	} 
	let mut _features_features_5_Add_output_0_1nscale = BN254Fr::default(); 
	if i_input._features_features_5_Add_output_0_1nscale >= 0{
		_features_features_5_Add_output_0_1nscale = BN254Fr::from((i_input._features_features_5_Add_output_0_1nscale) as u64); 
	} else {
		_features_features_5_Add_output_0_1nscale = -BN254Fr::from((-i_input._features_features_5_Add_output_0_1nscale) as u64); 
	} 
	let mut _features_features_5_Add_output_0_1dscale = BN254Fr::default(); 
	if i_input._features_features_5_Add_output_0_1dscale >= 0{
		_features_features_5_Add_output_0_1dscale = BN254Fr::from((i_input._features_features_5_Add_output_0_1dscale) as u64); 
	} else {
		_features_features_5_Add_output_0_1dscale = -BN254Fr::from((-i_input._features_features_5_Add_output_0_1dscale) as u64); 
	} 
	let mut _features_features_5_Add_output_0_2nscale = BN254Fr::default(); 
	if i_input._features_features_5_Add_output_0_2nscale >= 0{
		_features_features_5_Add_output_0_2nscale = BN254Fr::from((i_input._features_features_5_Add_output_0_2nscale) as u64); 
	} else {
		_features_features_5_Add_output_0_2nscale = -BN254Fr::from((-i_input._features_features_5_Add_output_0_2nscale) as u64); 
	} 
	let mut _features_features_5_Add_output_0_2dscale = BN254Fr::default(); 
	if i_input._features_features_5_Add_output_0_2dscale >= 0{
		_features_features_5_Add_output_0_2dscale = BN254Fr::from((i_input._features_features_5_Add_output_0_2dscale) as u64); 
	} else {
		_features_features_5_Add_output_0_2dscale = -BN254Fr::from((-i_input._features_features_5_Add_output_0_2dscale) as u64); 
	} 
	let mut onnx__Conv_666 = vec![vec![vec![vec![BN254Fr::default();1];1];32];192]; 
	for i in 0..192 {
		for j in 0..32 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_666[i][j][k][l] >= 0{
						onnx__Conv_666[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_666[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_666[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_666[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_667 = vec![BN254Fr::default();192]; 
	for i in 0..192 {
		if i_input.onnx__Conv_667[i] >= 0{
			onnx__Conv_667[i] = BN254Fr::from((i_input.onnx__Conv_667[i]) as u64); 
		} else {
			onnx__Conv_667[i] = -BN254Fr::from((-i_input.onnx__Conv_667[i]) as u64); 
		} 
	}
	let mut onnx__Conv_667_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_667_q[i][j][k] >= 0{
					onnx__Conv_667_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_667_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_667_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_667_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_666_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_666_nscale >= 0{
		onnx__Conv_666_nscale = BN254Fr::from((i_input.onnx__Conv_666_nscale) as u64); 
	} else {
		onnx__Conv_666_nscale = -BN254Fr::from((-i_input.onnx__Conv_666_nscale) as u64); 
	} 
	let mut onnx__Conv_666_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_666_dscale >= 0{
		onnx__Conv_666_dscale = BN254Fr::from((i_input.onnx__Conv_666_dscale) as u64); 
	} else {
		onnx__Conv_666_dscale = -BN254Fr::from((-i_input.onnx__Conv_666_dscale) as u64); 
	} 
	let mut onnx__PRelu_789_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_789_q[i][j][k] >= 0{
					onnx__PRelu_789_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_789_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_789_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_789_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_789_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_789_nscale >= 0{
		onnx__PRelu_789_nscale = BN254Fr::from((i_input.onnx__PRelu_789_nscale) as u64); 
	} else {
		onnx__PRelu_789_nscale = -BN254Fr::from((-i_input.onnx__PRelu_789_nscale) as u64); 
	} 
	let mut onnx__PRelu_789_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_789_dscale >= 0{
		onnx__PRelu_789_dscale = BN254Fr::from((i_input.onnx__PRelu_789_dscale) as u64); 
	} else {
		onnx__PRelu_789_dscale = -BN254Fr::from((-i_input.onnx__PRelu_789_dscale) as u64); 
	} 
	let mut onnx__PRelu_789_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_789_zero[i] >= 0{
			onnx__PRelu_789_zero[i] = BN254Fr::from((i_input.onnx__PRelu_789_zero[i]) as u64); 
		} else {
			onnx__PRelu_789_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_789_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_669 = vec![vec![vec![vec![BN254Fr::default();3];3];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_669[i][j][k][l] >= 0{
						onnx__Conv_669[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_669[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_669[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_669[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_670 = vec![BN254Fr::default();192]; 
	for i in 0..192 {
		if i_input.onnx__Conv_670[i] >= 0{
			onnx__Conv_670[i] = BN254Fr::from((i_input.onnx__Conv_670[i]) as u64); 
		} else {
			onnx__Conv_670[i] = -BN254Fr::from((-i_input.onnx__Conv_670[i]) as u64); 
		} 
	}
	let mut onnx__Conv_670_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_670_q[i][j][k] >= 0{
					onnx__Conv_670_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_670_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_670_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_670_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_669_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_669_nscale >= 0{
		onnx__Conv_669_nscale = BN254Fr::from((i_input.onnx__Conv_669_nscale) as u64); 
	} else {
		onnx__Conv_669_nscale = -BN254Fr::from((-i_input.onnx__Conv_669_nscale) as u64); 
	} 
	let mut onnx__Conv_669_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_669_dscale >= 0{
		onnx__Conv_669_dscale = BN254Fr::from((i_input.onnx__Conv_669_dscale) as u64); 
	} else {
		onnx__Conv_669_dscale = -BN254Fr::from((-i_input.onnx__Conv_669_dscale) as u64); 
	} 
	let mut onnx__PRelu_790_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_790_q[i][j][k] >= 0{
					onnx__PRelu_790_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_790_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_790_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_790_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_790_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_790_nscale >= 0{
		onnx__PRelu_790_nscale = BN254Fr::from((i_input.onnx__PRelu_790_nscale) as u64); 
	} else {
		onnx__PRelu_790_nscale = -BN254Fr::from((-i_input.onnx__PRelu_790_nscale) as u64); 
	} 
	let mut onnx__PRelu_790_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_790_dscale >= 0{
		onnx__PRelu_790_dscale = BN254Fr::from((i_input.onnx__PRelu_790_dscale) as u64); 
	} else {
		onnx__PRelu_790_dscale = -BN254Fr::from((-i_input.onnx__PRelu_790_dscale) as u64); 
	} 
	let mut onnx__PRelu_790_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_790_zero[i] >= 0{
			onnx__PRelu_790_zero[i] = BN254Fr::from((i_input.onnx__PRelu_790_zero[i]) as u64); 
		} else {
			onnx__PRelu_790_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_790_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_672 = vec![vec![vec![vec![BN254Fr::default();1];1];192];32]; 
	for i in 0..32 {
		for j in 0..192 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_672[i][j][k][l] >= 0{
						onnx__Conv_672[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_672[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_672[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_672[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_673 = vec![BN254Fr::default();32]; 
	for i in 0..32 {
		if i_input.onnx__Conv_673[i] >= 0{
			onnx__Conv_673[i] = BN254Fr::from((i_input.onnx__Conv_673[i]) as u64); 
		} else {
			onnx__Conv_673[i] = -BN254Fr::from((-i_input.onnx__Conv_673[i]) as u64); 
		} 
	}
	let mut onnx__Conv_673_q = vec![vec![vec![BN254Fr::default();1];1];32]; 
	for i in 0..32 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_673_q[i][j][k] >= 0{
					onnx__Conv_673_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_673_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_673_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_673_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_672_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_672_nscale >= 0{
		onnx__Conv_672_nscale = BN254Fr::from((i_input.onnx__Conv_672_nscale) as u64); 
	} else {
		onnx__Conv_672_nscale = -BN254Fr::from((-i_input.onnx__Conv_672_nscale) as u64); 
	} 
	let mut onnx__Conv_672_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_672_dscale >= 0{
		onnx__Conv_672_dscale = BN254Fr::from((i_input.onnx__Conv_672_dscale) as u64); 
	} else {
		onnx__Conv_672_dscale = -BN254Fr::from((-i_input.onnx__Conv_672_dscale) as u64); 
	} 
	let mut _features_features_6_Add_output_0_1nscale = BN254Fr::default(); 
	if i_input._features_features_6_Add_output_0_1nscale >= 0{
		_features_features_6_Add_output_0_1nscale = BN254Fr::from((i_input._features_features_6_Add_output_0_1nscale) as u64); 
	} else {
		_features_features_6_Add_output_0_1nscale = -BN254Fr::from((-i_input._features_features_6_Add_output_0_1nscale) as u64); 
	} 
	let mut _features_features_6_Add_output_0_1dscale = BN254Fr::default(); 
	if i_input._features_features_6_Add_output_0_1dscale >= 0{
		_features_features_6_Add_output_0_1dscale = BN254Fr::from((i_input._features_features_6_Add_output_0_1dscale) as u64); 
	} else {
		_features_features_6_Add_output_0_1dscale = -BN254Fr::from((-i_input._features_features_6_Add_output_0_1dscale) as u64); 
	} 
	let mut _features_features_6_Add_output_0_2nscale = BN254Fr::default(); 
	if i_input._features_features_6_Add_output_0_2nscale >= 0{
		_features_features_6_Add_output_0_2nscale = BN254Fr::from((i_input._features_features_6_Add_output_0_2nscale) as u64); 
	} else {
		_features_features_6_Add_output_0_2nscale = -BN254Fr::from((-i_input._features_features_6_Add_output_0_2nscale) as u64); 
	} 
	let mut _features_features_6_Add_output_0_2dscale = BN254Fr::default(); 
	if i_input._features_features_6_Add_output_0_2dscale >= 0{
		_features_features_6_Add_output_0_2dscale = BN254Fr::from((i_input._features_features_6_Add_output_0_2dscale) as u64); 
	} else {
		_features_features_6_Add_output_0_2dscale = -BN254Fr::from((-i_input._features_features_6_Add_output_0_2dscale) as u64); 
	} 
	let mut onnx__Conv_675 = vec![vec![vec![vec![BN254Fr::default();1];1];32];192]; 
	for i in 0..192 {
		for j in 0..32 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_675[i][j][k][l] >= 0{
						onnx__Conv_675[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_675[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_675[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_675[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_676 = vec![BN254Fr::default();192]; 
	for i in 0..192 {
		if i_input.onnx__Conv_676[i] >= 0{
			onnx__Conv_676[i] = BN254Fr::from((i_input.onnx__Conv_676[i]) as u64); 
		} else {
			onnx__Conv_676[i] = -BN254Fr::from((-i_input.onnx__Conv_676[i]) as u64); 
		} 
	}
	let mut onnx__Conv_676_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_676_q[i][j][k] >= 0{
					onnx__Conv_676_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_676_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_676_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_676_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_675_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_675_nscale >= 0{
		onnx__Conv_675_nscale = BN254Fr::from((i_input.onnx__Conv_675_nscale) as u64); 
	} else {
		onnx__Conv_675_nscale = -BN254Fr::from((-i_input.onnx__Conv_675_nscale) as u64); 
	} 
	let mut onnx__Conv_675_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_675_dscale >= 0{
		onnx__Conv_675_dscale = BN254Fr::from((i_input.onnx__Conv_675_dscale) as u64); 
	} else {
		onnx__Conv_675_dscale = -BN254Fr::from((-i_input.onnx__Conv_675_dscale) as u64); 
	} 
	let mut onnx__PRelu_791_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_791_q[i][j][k] >= 0{
					onnx__PRelu_791_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_791_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_791_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_791_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_791_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_791_nscale >= 0{
		onnx__PRelu_791_nscale = BN254Fr::from((i_input.onnx__PRelu_791_nscale) as u64); 
	} else {
		onnx__PRelu_791_nscale = -BN254Fr::from((-i_input.onnx__PRelu_791_nscale) as u64); 
	} 
	let mut onnx__PRelu_791_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_791_dscale >= 0{
		onnx__PRelu_791_dscale = BN254Fr::from((i_input.onnx__PRelu_791_dscale) as u64); 
	} else {
		onnx__PRelu_791_dscale = -BN254Fr::from((-i_input.onnx__PRelu_791_dscale) as u64); 
	} 
	let mut onnx__PRelu_791_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_791_zero[i] >= 0{
			onnx__PRelu_791_zero[i] = BN254Fr::from((i_input.onnx__PRelu_791_zero[i]) as u64); 
		} else {
			onnx__PRelu_791_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_791_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_678 = vec![vec![vec![vec![BN254Fr::default();3];3];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_678[i][j][k][l] >= 0{
						onnx__Conv_678[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_678[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_678[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_678[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_679 = vec![BN254Fr::default();192]; 
	for i in 0..192 {
		if i_input.onnx__Conv_679[i] >= 0{
			onnx__Conv_679[i] = BN254Fr::from((i_input.onnx__Conv_679[i]) as u64); 
		} else {
			onnx__Conv_679[i] = -BN254Fr::from((-i_input.onnx__Conv_679[i]) as u64); 
		} 
	}
	let mut onnx__Conv_679_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_679_q[i][j][k] >= 0{
					onnx__Conv_679_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_679_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_679_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_679_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_678_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_678_nscale >= 0{
		onnx__Conv_678_nscale = BN254Fr::from((i_input.onnx__Conv_678_nscale) as u64); 
	} else {
		onnx__Conv_678_nscale = -BN254Fr::from((-i_input.onnx__Conv_678_nscale) as u64); 
	} 
	let mut onnx__Conv_678_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_678_dscale >= 0{
		onnx__Conv_678_dscale = BN254Fr::from((i_input.onnx__Conv_678_dscale) as u64); 
	} else {
		onnx__Conv_678_dscale = -BN254Fr::from((-i_input.onnx__Conv_678_dscale) as u64); 
	} 
	let mut onnx__PRelu_792_q = vec![vec![vec![BN254Fr::default();1];1];192]; 
	for i in 0..192 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_792_q[i][j][k] >= 0{
					onnx__PRelu_792_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_792_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_792_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_792_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_792_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_792_nscale >= 0{
		onnx__PRelu_792_nscale = BN254Fr::from((i_input.onnx__PRelu_792_nscale) as u64); 
	} else {
		onnx__PRelu_792_nscale = -BN254Fr::from((-i_input.onnx__PRelu_792_nscale) as u64); 
	} 
	let mut onnx__PRelu_792_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_792_dscale >= 0{
		onnx__PRelu_792_dscale = BN254Fr::from((i_input.onnx__PRelu_792_dscale) as u64); 
	} else {
		onnx__PRelu_792_dscale = -BN254Fr::from((-i_input.onnx__PRelu_792_dscale) as u64); 
	} 
	let mut onnx__PRelu_792_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_792_zero[i] >= 0{
			onnx__PRelu_792_zero[i] = BN254Fr::from((i_input.onnx__PRelu_792_zero[i]) as u64); 
		} else {
			onnx__PRelu_792_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_792_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_681 = vec![vec![vec![vec![BN254Fr::default();1];1];192];64]; 
	for i in 0..64 {
		for j in 0..192 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_681[i][j][k][l] >= 0{
						onnx__Conv_681[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_681[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_681[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_681[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_682 = vec![BN254Fr::default();64]; 
	for i in 0..64 {
		if i_input.onnx__Conv_682[i] >= 0{
			onnx__Conv_682[i] = BN254Fr::from((i_input.onnx__Conv_682[i]) as u64); 
		} else {
			onnx__Conv_682[i] = -BN254Fr::from((-i_input.onnx__Conv_682[i]) as u64); 
		} 
	}
	let mut onnx__Conv_682_q = vec![vec![vec![BN254Fr::default();1];1];64]; 
	for i in 0..64 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_682_q[i][j][k] >= 0{
					onnx__Conv_682_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_682_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_682_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_682_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_681_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_681_nscale >= 0{
		onnx__Conv_681_nscale = BN254Fr::from((i_input.onnx__Conv_681_nscale) as u64); 
	} else {
		onnx__Conv_681_nscale = -BN254Fr::from((-i_input.onnx__Conv_681_nscale) as u64); 
	} 
	let mut onnx__Conv_681_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_681_dscale >= 0{
		onnx__Conv_681_dscale = BN254Fr::from((i_input.onnx__Conv_681_dscale) as u64); 
	} else {
		onnx__Conv_681_dscale = -BN254Fr::from((-i_input.onnx__Conv_681_dscale) as u64); 
	} 
	let mut onnx__Conv_684 = vec![vec![vec![vec![BN254Fr::default();1];1];64];384]; 
	for i in 0..384 {
		for j in 0..64 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_684[i][j][k][l] >= 0{
						onnx__Conv_684[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_684[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_684[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_684[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_685 = vec![BN254Fr::default();384]; 
	for i in 0..384 {
		if i_input.onnx__Conv_685[i] >= 0{
			onnx__Conv_685[i] = BN254Fr::from((i_input.onnx__Conv_685[i]) as u64); 
		} else {
			onnx__Conv_685[i] = -BN254Fr::from((-i_input.onnx__Conv_685[i]) as u64); 
		} 
	}
	let mut onnx__Conv_685_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_685_q[i][j][k] >= 0{
					onnx__Conv_685_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_685_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_685_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_685_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_684_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_684_nscale >= 0{
		onnx__Conv_684_nscale = BN254Fr::from((i_input.onnx__Conv_684_nscale) as u64); 
	} else {
		onnx__Conv_684_nscale = -BN254Fr::from((-i_input.onnx__Conv_684_nscale) as u64); 
	} 
	let mut onnx__Conv_684_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_684_dscale >= 0{
		onnx__Conv_684_dscale = BN254Fr::from((i_input.onnx__Conv_684_dscale) as u64); 
	} else {
		onnx__Conv_684_dscale = -BN254Fr::from((-i_input.onnx__Conv_684_dscale) as u64); 
	} 
	let mut onnx__PRelu_793_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_793_q[i][j][k] >= 0{
					onnx__PRelu_793_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_793_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_793_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_793_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_793_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_793_nscale >= 0{
		onnx__PRelu_793_nscale = BN254Fr::from((i_input.onnx__PRelu_793_nscale) as u64); 
	} else {
		onnx__PRelu_793_nscale = -BN254Fr::from((-i_input.onnx__PRelu_793_nscale) as u64); 
	} 
	let mut onnx__PRelu_793_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_793_dscale >= 0{
		onnx__PRelu_793_dscale = BN254Fr::from((i_input.onnx__PRelu_793_dscale) as u64); 
	} else {
		onnx__PRelu_793_dscale = -BN254Fr::from((-i_input.onnx__PRelu_793_dscale) as u64); 
	} 
	let mut onnx__PRelu_793_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_793_zero[i] >= 0{
			onnx__PRelu_793_zero[i] = BN254Fr::from((i_input.onnx__PRelu_793_zero[i]) as u64); 
		} else {
			onnx__PRelu_793_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_793_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_687 = vec![vec![vec![vec![BN254Fr::default();3];3];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_687[i][j][k][l] >= 0{
						onnx__Conv_687[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_687[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_687[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_687[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_688 = vec![BN254Fr::default();384]; 
	for i in 0..384 {
		if i_input.onnx__Conv_688[i] >= 0{
			onnx__Conv_688[i] = BN254Fr::from((i_input.onnx__Conv_688[i]) as u64); 
		} else {
			onnx__Conv_688[i] = -BN254Fr::from((-i_input.onnx__Conv_688[i]) as u64); 
		} 
	}
	let mut onnx__Conv_688_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_688_q[i][j][k] >= 0{
					onnx__Conv_688_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_688_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_688_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_688_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_687_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_687_nscale >= 0{
		onnx__Conv_687_nscale = BN254Fr::from((i_input.onnx__Conv_687_nscale) as u64); 
	} else {
		onnx__Conv_687_nscale = -BN254Fr::from((-i_input.onnx__Conv_687_nscale) as u64); 
	} 
	let mut onnx__Conv_687_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_687_dscale >= 0{
		onnx__Conv_687_dscale = BN254Fr::from((i_input.onnx__Conv_687_dscale) as u64); 
	} else {
		onnx__Conv_687_dscale = -BN254Fr::from((-i_input.onnx__Conv_687_dscale) as u64); 
	} 
	let mut onnx__PRelu_794_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_794_q[i][j][k] >= 0{
					onnx__PRelu_794_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_794_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_794_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_794_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_794_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_794_nscale >= 0{
		onnx__PRelu_794_nscale = BN254Fr::from((i_input.onnx__PRelu_794_nscale) as u64); 
	} else {
		onnx__PRelu_794_nscale = -BN254Fr::from((-i_input.onnx__PRelu_794_nscale) as u64); 
	} 
	let mut onnx__PRelu_794_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_794_dscale >= 0{
		onnx__PRelu_794_dscale = BN254Fr::from((i_input.onnx__PRelu_794_dscale) as u64); 
	} else {
		onnx__PRelu_794_dscale = -BN254Fr::from((-i_input.onnx__PRelu_794_dscale) as u64); 
	} 
	let mut onnx__PRelu_794_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_794_zero[i] >= 0{
			onnx__PRelu_794_zero[i] = BN254Fr::from((i_input.onnx__PRelu_794_zero[i]) as u64); 
		} else {
			onnx__PRelu_794_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_794_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_690 = vec![vec![vec![vec![BN254Fr::default();1];1];384];64]; 
	for i in 0..64 {
		for j in 0..384 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_690[i][j][k][l] >= 0{
						onnx__Conv_690[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_690[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_690[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_690[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_691 = vec![BN254Fr::default();64]; 
	for i in 0..64 {
		if i_input.onnx__Conv_691[i] >= 0{
			onnx__Conv_691[i] = BN254Fr::from((i_input.onnx__Conv_691[i]) as u64); 
		} else {
			onnx__Conv_691[i] = -BN254Fr::from((-i_input.onnx__Conv_691[i]) as u64); 
		} 
	}
	let mut onnx__Conv_691_q = vec![vec![vec![BN254Fr::default();1];1];64]; 
	for i in 0..64 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_691_q[i][j][k] >= 0{
					onnx__Conv_691_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_691_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_691_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_691_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_690_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_690_nscale >= 0{
		onnx__Conv_690_nscale = BN254Fr::from((i_input.onnx__Conv_690_nscale) as u64); 
	} else {
		onnx__Conv_690_nscale = -BN254Fr::from((-i_input.onnx__Conv_690_nscale) as u64); 
	} 
	let mut onnx__Conv_690_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_690_dscale >= 0{
		onnx__Conv_690_dscale = BN254Fr::from((i_input.onnx__Conv_690_dscale) as u64); 
	} else {
		onnx__Conv_690_dscale = -BN254Fr::from((-i_input.onnx__Conv_690_dscale) as u64); 
	} 
	let mut _features_features_8_Add_output_0_1nscale = BN254Fr::default(); 
	if i_input._features_features_8_Add_output_0_1nscale >= 0{
		_features_features_8_Add_output_0_1nscale = BN254Fr::from((i_input._features_features_8_Add_output_0_1nscale) as u64); 
	} else {
		_features_features_8_Add_output_0_1nscale = -BN254Fr::from((-i_input._features_features_8_Add_output_0_1nscale) as u64); 
	} 
	let mut _features_features_8_Add_output_0_1dscale = BN254Fr::default(); 
	if i_input._features_features_8_Add_output_0_1dscale >= 0{
		_features_features_8_Add_output_0_1dscale = BN254Fr::from((i_input._features_features_8_Add_output_0_1dscale) as u64); 
	} else {
		_features_features_8_Add_output_0_1dscale = -BN254Fr::from((-i_input._features_features_8_Add_output_0_1dscale) as u64); 
	} 
	let mut _features_features_8_Add_output_0_2nscale = BN254Fr::default(); 
	if i_input._features_features_8_Add_output_0_2nscale >= 0{
		_features_features_8_Add_output_0_2nscale = BN254Fr::from((i_input._features_features_8_Add_output_0_2nscale) as u64); 
	} else {
		_features_features_8_Add_output_0_2nscale = -BN254Fr::from((-i_input._features_features_8_Add_output_0_2nscale) as u64); 
	} 
	let mut _features_features_8_Add_output_0_2dscale = BN254Fr::default(); 
	if i_input._features_features_8_Add_output_0_2dscale >= 0{
		_features_features_8_Add_output_0_2dscale = BN254Fr::from((i_input._features_features_8_Add_output_0_2dscale) as u64); 
	} else {
		_features_features_8_Add_output_0_2dscale = -BN254Fr::from((-i_input._features_features_8_Add_output_0_2dscale) as u64); 
	} 
	let mut onnx__Conv_693 = vec![vec![vec![vec![BN254Fr::default();1];1];64];384]; 
	for i in 0..384 {
		for j in 0..64 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_693[i][j][k][l] >= 0{
						onnx__Conv_693[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_693[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_693[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_693[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_694 = vec![BN254Fr::default();384]; 
	for i in 0..384 {
		if i_input.onnx__Conv_694[i] >= 0{
			onnx__Conv_694[i] = BN254Fr::from((i_input.onnx__Conv_694[i]) as u64); 
		} else {
			onnx__Conv_694[i] = -BN254Fr::from((-i_input.onnx__Conv_694[i]) as u64); 
		} 
	}
	let mut onnx__Conv_694_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_694_q[i][j][k] >= 0{
					onnx__Conv_694_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_694_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_694_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_694_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_693_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_693_nscale >= 0{
		onnx__Conv_693_nscale = BN254Fr::from((i_input.onnx__Conv_693_nscale) as u64); 
	} else {
		onnx__Conv_693_nscale = -BN254Fr::from((-i_input.onnx__Conv_693_nscale) as u64); 
	} 
	let mut onnx__Conv_693_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_693_dscale >= 0{
		onnx__Conv_693_dscale = BN254Fr::from((i_input.onnx__Conv_693_dscale) as u64); 
	} else {
		onnx__Conv_693_dscale = -BN254Fr::from((-i_input.onnx__Conv_693_dscale) as u64); 
	} 
	let mut onnx__PRelu_795_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_795_q[i][j][k] >= 0{
					onnx__PRelu_795_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_795_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_795_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_795_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_795_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_795_nscale >= 0{
		onnx__PRelu_795_nscale = BN254Fr::from((i_input.onnx__PRelu_795_nscale) as u64); 
	} else {
		onnx__PRelu_795_nscale = -BN254Fr::from((-i_input.onnx__PRelu_795_nscale) as u64); 
	} 
	let mut onnx__PRelu_795_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_795_dscale >= 0{
		onnx__PRelu_795_dscale = BN254Fr::from((i_input.onnx__PRelu_795_dscale) as u64); 
	} else {
		onnx__PRelu_795_dscale = -BN254Fr::from((-i_input.onnx__PRelu_795_dscale) as u64); 
	} 
	let mut onnx__PRelu_795_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_795_zero[i] >= 0{
			onnx__PRelu_795_zero[i] = BN254Fr::from((i_input.onnx__PRelu_795_zero[i]) as u64); 
		} else {
			onnx__PRelu_795_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_795_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_696 = vec![vec![vec![vec![BN254Fr::default();3];3];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_696[i][j][k][l] >= 0{
						onnx__Conv_696[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_696[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_696[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_696[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_697 = vec![BN254Fr::default();384]; 
	for i in 0..384 {
		if i_input.onnx__Conv_697[i] >= 0{
			onnx__Conv_697[i] = BN254Fr::from((i_input.onnx__Conv_697[i]) as u64); 
		} else {
			onnx__Conv_697[i] = -BN254Fr::from((-i_input.onnx__Conv_697[i]) as u64); 
		} 
	}
	let mut onnx__Conv_697_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_697_q[i][j][k] >= 0{
					onnx__Conv_697_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_697_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_697_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_697_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_696_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_696_nscale >= 0{
		onnx__Conv_696_nscale = BN254Fr::from((i_input.onnx__Conv_696_nscale) as u64); 
	} else {
		onnx__Conv_696_nscale = -BN254Fr::from((-i_input.onnx__Conv_696_nscale) as u64); 
	} 
	let mut onnx__Conv_696_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_696_dscale >= 0{
		onnx__Conv_696_dscale = BN254Fr::from((i_input.onnx__Conv_696_dscale) as u64); 
	} else {
		onnx__Conv_696_dscale = -BN254Fr::from((-i_input.onnx__Conv_696_dscale) as u64); 
	} 
	let mut onnx__PRelu_796_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_796_q[i][j][k] >= 0{
					onnx__PRelu_796_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_796_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_796_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_796_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_796_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_796_nscale >= 0{
		onnx__PRelu_796_nscale = BN254Fr::from((i_input.onnx__PRelu_796_nscale) as u64); 
	} else {
		onnx__PRelu_796_nscale = -BN254Fr::from((-i_input.onnx__PRelu_796_nscale) as u64); 
	} 
	let mut onnx__PRelu_796_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_796_dscale >= 0{
		onnx__PRelu_796_dscale = BN254Fr::from((i_input.onnx__PRelu_796_dscale) as u64); 
	} else {
		onnx__PRelu_796_dscale = -BN254Fr::from((-i_input.onnx__PRelu_796_dscale) as u64); 
	} 
	let mut onnx__PRelu_796_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_796_zero[i] >= 0{
			onnx__PRelu_796_zero[i] = BN254Fr::from((i_input.onnx__PRelu_796_zero[i]) as u64); 
		} else {
			onnx__PRelu_796_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_796_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_699 = vec![vec![vec![vec![BN254Fr::default();1];1];384];64]; 
	for i in 0..64 {
		for j in 0..384 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_699[i][j][k][l] >= 0{
						onnx__Conv_699[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_699[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_699[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_699[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_700 = vec![BN254Fr::default();64]; 
	for i in 0..64 {
		if i_input.onnx__Conv_700[i] >= 0{
			onnx__Conv_700[i] = BN254Fr::from((i_input.onnx__Conv_700[i]) as u64); 
		} else {
			onnx__Conv_700[i] = -BN254Fr::from((-i_input.onnx__Conv_700[i]) as u64); 
		} 
	}
	let mut onnx__Conv_700_q = vec![vec![vec![BN254Fr::default();1];1];64]; 
	for i in 0..64 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_700_q[i][j][k] >= 0{
					onnx__Conv_700_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_700_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_700_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_700_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_699_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_699_nscale >= 0{
		onnx__Conv_699_nscale = BN254Fr::from((i_input.onnx__Conv_699_nscale) as u64); 
	} else {
		onnx__Conv_699_nscale = -BN254Fr::from((-i_input.onnx__Conv_699_nscale) as u64); 
	} 
	let mut onnx__Conv_699_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_699_dscale >= 0{
		onnx__Conv_699_dscale = BN254Fr::from((i_input.onnx__Conv_699_dscale) as u64); 
	} else {
		onnx__Conv_699_dscale = -BN254Fr::from((-i_input.onnx__Conv_699_dscale) as u64); 
	} 
	let mut _features_features_9_Add_output_0_1nscale = BN254Fr::default(); 
	if i_input._features_features_9_Add_output_0_1nscale >= 0{
		_features_features_9_Add_output_0_1nscale = BN254Fr::from((i_input._features_features_9_Add_output_0_1nscale) as u64); 
	} else {
		_features_features_9_Add_output_0_1nscale = -BN254Fr::from((-i_input._features_features_9_Add_output_0_1nscale) as u64); 
	} 
	let mut _features_features_9_Add_output_0_1dscale = BN254Fr::default(); 
	if i_input._features_features_9_Add_output_0_1dscale >= 0{
		_features_features_9_Add_output_0_1dscale = BN254Fr::from((i_input._features_features_9_Add_output_0_1dscale) as u64); 
	} else {
		_features_features_9_Add_output_0_1dscale = -BN254Fr::from((-i_input._features_features_9_Add_output_0_1dscale) as u64); 
	} 
	let mut _features_features_9_Add_output_0_2nscale = BN254Fr::default(); 
	if i_input._features_features_9_Add_output_0_2nscale >= 0{
		_features_features_9_Add_output_0_2nscale = BN254Fr::from((i_input._features_features_9_Add_output_0_2nscale) as u64); 
	} else {
		_features_features_9_Add_output_0_2nscale = -BN254Fr::from((-i_input._features_features_9_Add_output_0_2nscale) as u64); 
	} 
	let mut _features_features_9_Add_output_0_2dscale = BN254Fr::default(); 
	if i_input._features_features_9_Add_output_0_2dscale >= 0{
		_features_features_9_Add_output_0_2dscale = BN254Fr::from((i_input._features_features_9_Add_output_0_2dscale) as u64); 
	} else {
		_features_features_9_Add_output_0_2dscale = -BN254Fr::from((-i_input._features_features_9_Add_output_0_2dscale) as u64); 
	} 
	let mut onnx__Conv_702 = vec![vec![vec![vec![BN254Fr::default();1];1];64];384]; 
	for i in 0..384 {
		for j in 0..64 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_702[i][j][k][l] >= 0{
						onnx__Conv_702[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_702[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_702[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_702[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_703 = vec![BN254Fr::default();384]; 
	for i in 0..384 {
		if i_input.onnx__Conv_703[i] >= 0{
			onnx__Conv_703[i] = BN254Fr::from((i_input.onnx__Conv_703[i]) as u64); 
		} else {
			onnx__Conv_703[i] = -BN254Fr::from((-i_input.onnx__Conv_703[i]) as u64); 
		} 
	}
	let mut onnx__Conv_703_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_703_q[i][j][k] >= 0{
					onnx__Conv_703_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_703_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_703_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_703_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_702_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_702_nscale >= 0{
		onnx__Conv_702_nscale = BN254Fr::from((i_input.onnx__Conv_702_nscale) as u64); 
	} else {
		onnx__Conv_702_nscale = -BN254Fr::from((-i_input.onnx__Conv_702_nscale) as u64); 
	} 
	let mut onnx__Conv_702_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_702_dscale >= 0{
		onnx__Conv_702_dscale = BN254Fr::from((i_input.onnx__Conv_702_dscale) as u64); 
	} else {
		onnx__Conv_702_dscale = -BN254Fr::from((-i_input.onnx__Conv_702_dscale) as u64); 
	} 
	let mut onnx__PRelu_797_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_797_q[i][j][k] >= 0{
					onnx__PRelu_797_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_797_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_797_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_797_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_797_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_797_nscale >= 0{
		onnx__PRelu_797_nscale = BN254Fr::from((i_input.onnx__PRelu_797_nscale) as u64); 
	} else {
		onnx__PRelu_797_nscale = -BN254Fr::from((-i_input.onnx__PRelu_797_nscale) as u64); 
	} 
	let mut onnx__PRelu_797_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_797_dscale >= 0{
		onnx__PRelu_797_dscale = BN254Fr::from((i_input.onnx__PRelu_797_dscale) as u64); 
	} else {
		onnx__PRelu_797_dscale = -BN254Fr::from((-i_input.onnx__PRelu_797_dscale) as u64); 
	} 
	let mut onnx__PRelu_797_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_797_zero[i] >= 0{
			onnx__PRelu_797_zero[i] = BN254Fr::from((i_input.onnx__PRelu_797_zero[i]) as u64); 
		} else {
			onnx__PRelu_797_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_797_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_705 = vec![vec![vec![vec![BN254Fr::default();3];3];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_705[i][j][k][l] >= 0{
						onnx__Conv_705[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_705[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_705[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_705[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_706 = vec![BN254Fr::default();384]; 
	for i in 0..384 {
		if i_input.onnx__Conv_706[i] >= 0{
			onnx__Conv_706[i] = BN254Fr::from((i_input.onnx__Conv_706[i]) as u64); 
		} else {
			onnx__Conv_706[i] = -BN254Fr::from((-i_input.onnx__Conv_706[i]) as u64); 
		} 
	}
	let mut onnx__Conv_706_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_706_q[i][j][k] >= 0{
					onnx__Conv_706_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_706_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_706_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_706_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_705_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_705_nscale >= 0{
		onnx__Conv_705_nscale = BN254Fr::from((i_input.onnx__Conv_705_nscale) as u64); 
	} else {
		onnx__Conv_705_nscale = -BN254Fr::from((-i_input.onnx__Conv_705_nscale) as u64); 
	} 
	let mut onnx__Conv_705_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_705_dscale >= 0{
		onnx__Conv_705_dscale = BN254Fr::from((i_input.onnx__Conv_705_dscale) as u64); 
	} else {
		onnx__Conv_705_dscale = -BN254Fr::from((-i_input.onnx__Conv_705_dscale) as u64); 
	} 
	let mut onnx__PRelu_798_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_798_q[i][j][k] >= 0{
					onnx__PRelu_798_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_798_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_798_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_798_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_798_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_798_nscale >= 0{
		onnx__PRelu_798_nscale = BN254Fr::from((i_input.onnx__PRelu_798_nscale) as u64); 
	} else {
		onnx__PRelu_798_nscale = -BN254Fr::from((-i_input.onnx__PRelu_798_nscale) as u64); 
	} 
	let mut onnx__PRelu_798_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_798_dscale >= 0{
		onnx__PRelu_798_dscale = BN254Fr::from((i_input.onnx__PRelu_798_dscale) as u64); 
	} else {
		onnx__PRelu_798_dscale = -BN254Fr::from((-i_input.onnx__PRelu_798_dscale) as u64); 
	} 
	let mut onnx__PRelu_798_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_798_zero[i] >= 0{
			onnx__PRelu_798_zero[i] = BN254Fr::from((i_input.onnx__PRelu_798_zero[i]) as u64); 
		} else {
			onnx__PRelu_798_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_798_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_708 = vec![vec![vec![vec![BN254Fr::default();1];1];384];64]; 
	for i in 0..64 {
		for j in 0..384 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_708[i][j][k][l] >= 0{
						onnx__Conv_708[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_708[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_708[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_708[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_709 = vec![BN254Fr::default();64]; 
	for i in 0..64 {
		if i_input.onnx__Conv_709[i] >= 0{
			onnx__Conv_709[i] = BN254Fr::from((i_input.onnx__Conv_709[i]) as u64); 
		} else {
			onnx__Conv_709[i] = -BN254Fr::from((-i_input.onnx__Conv_709[i]) as u64); 
		} 
	}
	let mut onnx__Conv_709_q = vec![vec![vec![BN254Fr::default();1];1];64]; 
	for i in 0..64 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_709_q[i][j][k] >= 0{
					onnx__Conv_709_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_709_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_709_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_709_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_708_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_708_nscale >= 0{
		onnx__Conv_708_nscale = BN254Fr::from((i_input.onnx__Conv_708_nscale) as u64); 
	} else {
		onnx__Conv_708_nscale = -BN254Fr::from((-i_input.onnx__Conv_708_nscale) as u64); 
	} 
	let mut onnx__Conv_708_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_708_dscale >= 0{
		onnx__Conv_708_dscale = BN254Fr::from((i_input.onnx__Conv_708_dscale) as u64); 
	} else {
		onnx__Conv_708_dscale = -BN254Fr::from((-i_input.onnx__Conv_708_dscale) as u64); 
	} 
	let mut _features_features_10_Add_output_0_1nscale = BN254Fr::default(); 
	if i_input._features_features_10_Add_output_0_1nscale >= 0{
		_features_features_10_Add_output_0_1nscale = BN254Fr::from((i_input._features_features_10_Add_output_0_1nscale) as u64); 
	} else {
		_features_features_10_Add_output_0_1nscale = -BN254Fr::from((-i_input._features_features_10_Add_output_0_1nscale) as u64); 
	} 
	let mut _features_features_10_Add_output_0_1dscale = BN254Fr::default(); 
	if i_input._features_features_10_Add_output_0_1dscale >= 0{
		_features_features_10_Add_output_0_1dscale = BN254Fr::from((i_input._features_features_10_Add_output_0_1dscale) as u64); 
	} else {
		_features_features_10_Add_output_0_1dscale = -BN254Fr::from((-i_input._features_features_10_Add_output_0_1dscale) as u64); 
	} 
	let mut _features_features_10_Add_output_0_2nscale = BN254Fr::default(); 
	if i_input._features_features_10_Add_output_0_2nscale >= 0{
		_features_features_10_Add_output_0_2nscale = BN254Fr::from((i_input._features_features_10_Add_output_0_2nscale) as u64); 
	} else {
		_features_features_10_Add_output_0_2nscale = -BN254Fr::from((-i_input._features_features_10_Add_output_0_2nscale) as u64); 
	} 
	let mut _features_features_10_Add_output_0_2dscale = BN254Fr::default(); 
	if i_input._features_features_10_Add_output_0_2dscale >= 0{
		_features_features_10_Add_output_0_2dscale = BN254Fr::from((i_input._features_features_10_Add_output_0_2dscale) as u64); 
	} else {
		_features_features_10_Add_output_0_2dscale = -BN254Fr::from((-i_input._features_features_10_Add_output_0_2dscale) as u64); 
	} 
	let mut onnx__Conv_711 = vec![vec![vec![vec![BN254Fr::default();1];1];64];384]; 
	for i in 0..384 {
		for j in 0..64 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_711[i][j][k][l] >= 0{
						onnx__Conv_711[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_711[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_711[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_711[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_712 = vec![BN254Fr::default();384]; 
	for i in 0..384 {
		if i_input.onnx__Conv_712[i] >= 0{
			onnx__Conv_712[i] = BN254Fr::from((i_input.onnx__Conv_712[i]) as u64); 
		} else {
			onnx__Conv_712[i] = -BN254Fr::from((-i_input.onnx__Conv_712[i]) as u64); 
		} 
	}
	let mut onnx__Conv_712_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_712_q[i][j][k] >= 0{
					onnx__Conv_712_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_712_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_712_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_712_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_711_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_711_nscale >= 0{
		onnx__Conv_711_nscale = BN254Fr::from((i_input.onnx__Conv_711_nscale) as u64); 
	} else {
		onnx__Conv_711_nscale = -BN254Fr::from((-i_input.onnx__Conv_711_nscale) as u64); 
	} 
	let mut onnx__Conv_711_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_711_dscale >= 0{
		onnx__Conv_711_dscale = BN254Fr::from((i_input.onnx__Conv_711_dscale) as u64); 
	} else {
		onnx__Conv_711_dscale = -BN254Fr::from((-i_input.onnx__Conv_711_dscale) as u64); 
	} 
	let mut onnx__PRelu_799_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_799_q[i][j][k] >= 0{
					onnx__PRelu_799_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_799_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_799_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_799_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_799_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_799_nscale >= 0{
		onnx__PRelu_799_nscale = BN254Fr::from((i_input.onnx__PRelu_799_nscale) as u64); 
	} else {
		onnx__PRelu_799_nscale = -BN254Fr::from((-i_input.onnx__PRelu_799_nscale) as u64); 
	} 
	let mut onnx__PRelu_799_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_799_dscale >= 0{
		onnx__PRelu_799_dscale = BN254Fr::from((i_input.onnx__PRelu_799_dscale) as u64); 
	} else {
		onnx__PRelu_799_dscale = -BN254Fr::from((-i_input.onnx__PRelu_799_dscale) as u64); 
	} 
	let mut onnx__PRelu_799_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_799_zero[i] >= 0{
			onnx__PRelu_799_zero[i] = BN254Fr::from((i_input.onnx__PRelu_799_zero[i]) as u64); 
		} else {
			onnx__PRelu_799_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_799_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_714 = vec![vec![vec![vec![BN254Fr::default();3];3];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_714[i][j][k][l] >= 0{
						onnx__Conv_714[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_714[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_714[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_714[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_715 = vec![BN254Fr::default();384]; 
	for i in 0..384 {
		if i_input.onnx__Conv_715[i] >= 0{
			onnx__Conv_715[i] = BN254Fr::from((i_input.onnx__Conv_715[i]) as u64); 
		} else {
			onnx__Conv_715[i] = -BN254Fr::from((-i_input.onnx__Conv_715[i]) as u64); 
		} 
	}
	let mut onnx__Conv_715_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_715_q[i][j][k] >= 0{
					onnx__Conv_715_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_715_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_715_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_715_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_714_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_714_nscale >= 0{
		onnx__Conv_714_nscale = BN254Fr::from((i_input.onnx__Conv_714_nscale) as u64); 
	} else {
		onnx__Conv_714_nscale = -BN254Fr::from((-i_input.onnx__Conv_714_nscale) as u64); 
	} 
	let mut onnx__Conv_714_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_714_dscale >= 0{
		onnx__Conv_714_dscale = BN254Fr::from((i_input.onnx__Conv_714_dscale) as u64); 
	} else {
		onnx__Conv_714_dscale = -BN254Fr::from((-i_input.onnx__Conv_714_dscale) as u64); 
	} 
	let mut onnx__PRelu_800_q = vec![vec![vec![BN254Fr::default();1];1];384]; 
	for i in 0..384 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_800_q[i][j][k] >= 0{
					onnx__PRelu_800_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_800_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_800_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_800_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_800_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_800_nscale >= 0{
		onnx__PRelu_800_nscale = BN254Fr::from((i_input.onnx__PRelu_800_nscale) as u64); 
	} else {
		onnx__PRelu_800_nscale = -BN254Fr::from((-i_input.onnx__PRelu_800_nscale) as u64); 
	} 
	let mut onnx__PRelu_800_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_800_dscale >= 0{
		onnx__PRelu_800_dscale = BN254Fr::from((i_input.onnx__PRelu_800_dscale) as u64); 
	} else {
		onnx__PRelu_800_dscale = -BN254Fr::from((-i_input.onnx__PRelu_800_dscale) as u64); 
	} 
	let mut onnx__PRelu_800_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_800_zero[i] >= 0{
			onnx__PRelu_800_zero[i] = BN254Fr::from((i_input.onnx__PRelu_800_zero[i]) as u64); 
		} else {
			onnx__PRelu_800_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_800_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_717 = vec![vec![vec![vec![BN254Fr::default();1];1];384];96]; 
	for i in 0..96 {
		for j in 0..384 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_717[i][j][k][l] >= 0{
						onnx__Conv_717[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_717[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_717[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_717[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_718 = vec![BN254Fr::default();96]; 
	for i in 0..96 {
		if i_input.onnx__Conv_718[i] >= 0{
			onnx__Conv_718[i] = BN254Fr::from((i_input.onnx__Conv_718[i]) as u64); 
		} else {
			onnx__Conv_718[i] = -BN254Fr::from((-i_input.onnx__Conv_718[i]) as u64); 
		} 
	}
	let mut onnx__Conv_718_q = vec![vec![vec![BN254Fr::default();1];1];96]; 
	for i in 0..96 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_718_q[i][j][k] >= 0{
					onnx__Conv_718_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_718_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_718_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_718_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_717_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_717_nscale >= 0{
		onnx__Conv_717_nscale = BN254Fr::from((i_input.onnx__Conv_717_nscale) as u64); 
	} else {
		onnx__Conv_717_nscale = -BN254Fr::from((-i_input.onnx__Conv_717_nscale) as u64); 
	} 
	let mut onnx__Conv_717_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_717_dscale >= 0{
		onnx__Conv_717_dscale = BN254Fr::from((i_input.onnx__Conv_717_dscale) as u64); 
	} else {
		onnx__Conv_717_dscale = -BN254Fr::from((-i_input.onnx__Conv_717_dscale) as u64); 
	} 
	let mut onnx__Conv_720 = vec![vec![vec![vec![BN254Fr::default();1];1];96];576]; 
	for i in 0..576 {
		for j in 0..96 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_720[i][j][k][l] >= 0{
						onnx__Conv_720[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_720[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_720[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_720[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_721 = vec![BN254Fr::default();576]; 
	for i in 0..576 {
		if i_input.onnx__Conv_721[i] >= 0{
			onnx__Conv_721[i] = BN254Fr::from((i_input.onnx__Conv_721[i]) as u64); 
		} else {
			onnx__Conv_721[i] = -BN254Fr::from((-i_input.onnx__Conv_721[i]) as u64); 
		} 
	}
	let mut onnx__Conv_721_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_721_q[i][j][k] >= 0{
					onnx__Conv_721_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_721_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_721_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_721_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_720_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_720_nscale >= 0{
		onnx__Conv_720_nscale = BN254Fr::from((i_input.onnx__Conv_720_nscale) as u64); 
	} else {
		onnx__Conv_720_nscale = -BN254Fr::from((-i_input.onnx__Conv_720_nscale) as u64); 
	} 
	let mut onnx__Conv_720_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_720_dscale >= 0{
		onnx__Conv_720_dscale = BN254Fr::from((i_input.onnx__Conv_720_dscale) as u64); 
	} else {
		onnx__Conv_720_dscale = -BN254Fr::from((-i_input.onnx__Conv_720_dscale) as u64); 
	} 
	let mut onnx__PRelu_801_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_801_q[i][j][k] >= 0{
					onnx__PRelu_801_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_801_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_801_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_801_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_801_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_801_nscale >= 0{
		onnx__PRelu_801_nscale = BN254Fr::from((i_input.onnx__PRelu_801_nscale) as u64); 
	} else {
		onnx__PRelu_801_nscale = -BN254Fr::from((-i_input.onnx__PRelu_801_nscale) as u64); 
	} 
	let mut onnx__PRelu_801_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_801_dscale >= 0{
		onnx__PRelu_801_dscale = BN254Fr::from((i_input.onnx__PRelu_801_dscale) as u64); 
	} else {
		onnx__PRelu_801_dscale = -BN254Fr::from((-i_input.onnx__PRelu_801_dscale) as u64); 
	} 
	let mut onnx__PRelu_801_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_801_zero[i] >= 0{
			onnx__PRelu_801_zero[i] = BN254Fr::from((i_input.onnx__PRelu_801_zero[i]) as u64); 
		} else {
			onnx__PRelu_801_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_801_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_723 = vec![vec![vec![vec![BN254Fr::default();3];3];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_723[i][j][k][l] >= 0{
						onnx__Conv_723[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_723[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_723[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_723[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_724 = vec![BN254Fr::default();576]; 
	for i in 0..576 {
		if i_input.onnx__Conv_724[i] >= 0{
			onnx__Conv_724[i] = BN254Fr::from((i_input.onnx__Conv_724[i]) as u64); 
		} else {
			onnx__Conv_724[i] = -BN254Fr::from((-i_input.onnx__Conv_724[i]) as u64); 
		} 
	}
	let mut onnx__Conv_724_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_724_q[i][j][k] >= 0{
					onnx__Conv_724_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_724_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_724_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_724_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_723_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_723_nscale >= 0{
		onnx__Conv_723_nscale = BN254Fr::from((i_input.onnx__Conv_723_nscale) as u64); 
	} else {
		onnx__Conv_723_nscale = -BN254Fr::from((-i_input.onnx__Conv_723_nscale) as u64); 
	} 
	let mut onnx__Conv_723_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_723_dscale >= 0{
		onnx__Conv_723_dscale = BN254Fr::from((i_input.onnx__Conv_723_dscale) as u64); 
	} else {
		onnx__Conv_723_dscale = -BN254Fr::from((-i_input.onnx__Conv_723_dscale) as u64); 
	} 
	let mut onnx__PRelu_802_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_802_q[i][j][k] >= 0{
					onnx__PRelu_802_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_802_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_802_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_802_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_802_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_802_nscale >= 0{
		onnx__PRelu_802_nscale = BN254Fr::from((i_input.onnx__PRelu_802_nscale) as u64); 
	} else {
		onnx__PRelu_802_nscale = -BN254Fr::from((-i_input.onnx__PRelu_802_nscale) as u64); 
	} 
	let mut onnx__PRelu_802_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_802_dscale >= 0{
		onnx__PRelu_802_dscale = BN254Fr::from((i_input.onnx__PRelu_802_dscale) as u64); 
	} else {
		onnx__PRelu_802_dscale = -BN254Fr::from((-i_input.onnx__PRelu_802_dscale) as u64); 
	} 
	let mut onnx__PRelu_802_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_802_zero[i] >= 0{
			onnx__PRelu_802_zero[i] = BN254Fr::from((i_input.onnx__PRelu_802_zero[i]) as u64); 
		} else {
			onnx__PRelu_802_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_802_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_726 = vec![vec![vec![vec![BN254Fr::default();1];1];576];96]; 
	for i in 0..96 {
		for j in 0..576 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_726[i][j][k][l] >= 0{
						onnx__Conv_726[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_726[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_726[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_726[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_727 = vec![BN254Fr::default();96]; 
	for i in 0..96 {
		if i_input.onnx__Conv_727[i] >= 0{
			onnx__Conv_727[i] = BN254Fr::from((i_input.onnx__Conv_727[i]) as u64); 
		} else {
			onnx__Conv_727[i] = -BN254Fr::from((-i_input.onnx__Conv_727[i]) as u64); 
		} 
	}
	let mut onnx__Conv_727_q = vec![vec![vec![BN254Fr::default();1];1];96]; 
	for i in 0..96 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_727_q[i][j][k] >= 0{
					onnx__Conv_727_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_727_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_727_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_727_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_726_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_726_nscale >= 0{
		onnx__Conv_726_nscale = BN254Fr::from((i_input.onnx__Conv_726_nscale) as u64); 
	} else {
		onnx__Conv_726_nscale = -BN254Fr::from((-i_input.onnx__Conv_726_nscale) as u64); 
	} 
	let mut onnx__Conv_726_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_726_dscale >= 0{
		onnx__Conv_726_dscale = BN254Fr::from((i_input.onnx__Conv_726_dscale) as u64); 
	} else {
		onnx__Conv_726_dscale = -BN254Fr::from((-i_input.onnx__Conv_726_dscale) as u64); 
	} 
	let mut _features_features_12_Add_output_0_1nscale = BN254Fr::default(); 
	if i_input._features_features_12_Add_output_0_1nscale >= 0{
		_features_features_12_Add_output_0_1nscale = BN254Fr::from((i_input._features_features_12_Add_output_0_1nscale) as u64); 
	} else {
		_features_features_12_Add_output_0_1nscale = -BN254Fr::from((-i_input._features_features_12_Add_output_0_1nscale) as u64); 
	} 
	let mut _features_features_12_Add_output_0_1dscale = BN254Fr::default(); 
	if i_input._features_features_12_Add_output_0_1dscale >= 0{
		_features_features_12_Add_output_0_1dscale = BN254Fr::from((i_input._features_features_12_Add_output_0_1dscale) as u64); 
	} else {
		_features_features_12_Add_output_0_1dscale = -BN254Fr::from((-i_input._features_features_12_Add_output_0_1dscale) as u64); 
	} 
	let mut _features_features_12_Add_output_0_2nscale = BN254Fr::default(); 
	if i_input._features_features_12_Add_output_0_2nscale >= 0{
		_features_features_12_Add_output_0_2nscale = BN254Fr::from((i_input._features_features_12_Add_output_0_2nscale) as u64); 
	} else {
		_features_features_12_Add_output_0_2nscale = -BN254Fr::from((-i_input._features_features_12_Add_output_0_2nscale) as u64); 
	} 
	let mut _features_features_12_Add_output_0_2dscale = BN254Fr::default(); 
	if i_input._features_features_12_Add_output_0_2dscale >= 0{
		_features_features_12_Add_output_0_2dscale = BN254Fr::from((i_input._features_features_12_Add_output_0_2dscale) as u64); 
	} else {
		_features_features_12_Add_output_0_2dscale = -BN254Fr::from((-i_input._features_features_12_Add_output_0_2dscale) as u64); 
	} 
	let mut onnx__Conv_729 = vec![vec![vec![vec![BN254Fr::default();1];1];96];576]; 
	for i in 0..576 {
		for j in 0..96 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_729[i][j][k][l] >= 0{
						onnx__Conv_729[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_729[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_729[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_729[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_730 = vec![BN254Fr::default();576]; 
	for i in 0..576 {
		if i_input.onnx__Conv_730[i] >= 0{
			onnx__Conv_730[i] = BN254Fr::from((i_input.onnx__Conv_730[i]) as u64); 
		} else {
			onnx__Conv_730[i] = -BN254Fr::from((-i_input.onnx__Conv_730[i]) as u64); 
		} 
	}
	let mut onnx__Conv_730_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_730_q[i][j][k] >= 0{
					onnx__Conv_730_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_730_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_730_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_730_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_729_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_729_nscale >= 0{
		onnx__Conv_729_nscale = BN254Fr::from((i_input.onnx__Conv_729_nscale) as u64); 
	} else {
		onnx__Conv_729_nscale = -BN254Fr::from((-i_input.onnx__Conv_729_nscale) as u64); 
	} 
	let mut onnx__Conv_729_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_729_dscale >= 0{
		onnx__Conv_729_dscale = BN254Fr::from((i_input.onnx__Conv_729_dscale) as u64); 
	} else {
		onnx__Conv_729_dscale = -BN254Fr::from((-i_input.onnx__Conv_729_dscale) as u64); 
	} 
	let mut onnx__PRelu_803_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_803_q[i][j][k] >= 0{
					onnx__PRelu_803_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_803_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_803_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_803_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_803_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_803_nscale >= 0{
		onnx__PRelu_803_nscale = BN254Fr::from((i_input.onnx__PRelu_803_nscale) as u64); 
	} else {
		onnx__PRelu_803_nscale = -BN254Fr::from((-i_input.onnx__PRelu_803_nscale) as u64); 
	} 
	let mut onnx__PRelu_803_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_803_dscale >= 0{
		onnx__PRelu_803_dscale = BN254Fr::from((i_input.onnx__PRelu_803_dscale) as u64); 
	} else {
		onnx__PRelu_803_dscale = -BN254Fr::from((-i_input.onnx__PRelu_803_dscale) as u64); 
	} 
	let mut onnx__PRelu_803_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_803_zero[i] >= 0{
			onnx__PRelu_803_zero[i] = BN254Fr::from((i_input.onnx__PRelu_803_zero[i]) as u64); 
		} else {
			onnx__PRelu_803_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_803_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_732 = vec![vec![vec![vec![BN254Fr::default();3];3];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_732[i][j][k][l] >= 0{
						onnx__Conv_732[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_732[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_732[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_732[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_733 = vec![BN254Fr::default();576]; 
	for i in 0..576 {
		if i_input.onnx__Conv_733[i] >= 0{
			onnx__Conv_733[i] = BN254Fr::from((i_input.onnx__Conv_733[i]) as u64); 
		} else {
			onnx__Conv_733[i] = -BN254Fr::from((-i_input.onnx__Conv_733[i]) as u64); 
		} 
	}
	let mut onnx__Conv_733_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_733_q[i][j][k] >= 0{
					onnx__Conv_733_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_733_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_733_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_733_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_732_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_732_nscale >= 0{
		onnx__Conv_732_nscale = BN254Fr::from((i_input.onnx__Conv_732_nscale) as u64); 
	} else {
		onnx__Conv_732_nscale = -BN254Fr::from((-i_input.onnx__Conv_732_nscale) as u64); 
	} 
	let mut onnx__Conv_732_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_732_dscale >= 0{
		onnx__Conv_732_dscale = BN254Fr::from((i_input.onnx__Conv_732_dscale) as u64); 
	} else {
		onnx__Conv_732_dscale = -BN254Fr::from((-i_input.onnx__Conv_732_dscale) as u64); 
	} 
	let mut onnx__PRelu_804_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_804_q[i][j][k] >= 0{
					onnx__PRelu_804_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_804_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_804_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_804_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_804_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_804_nscale >= 0{
		onnx__PRelu_804_nscale = BN254Fr::from((i_input.onnx__PRelu_804_nscale) as u64); 
	} else {
		onnx__PRelu_804_nscale = -BN254Fr::from((-i_input.onnx__PRelu_804_nscale) as u64); 
	} 
	let mut onnx__PRelu_804_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_804_dscale >= 0{
		onnx__PRelu_804_dscale = BN254Fr::from((i_input.onnx__PRelu_804_dscale) as u64); 
	} else {
		onnx__PRelu_804_dscale = -BN254Fr::from((-i_input.onnx__PRelu_804_dscale) as u64); 
	} 
	let mut onnx__PRelu_804_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_804_zero[i] >= 0{
			onnx__PRelu_804_zero[i] = BN254Fr::from((i_input.onnx__PRelu_804_zero[i]) as u64); 
		} else {
			onnx__PRelu_804_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_804_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_735 = vec![vec![vec![vec![BN254Fr::default();1];1];576];96]; 
	for i in 0..96 {
		for j in 0..576 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_735[i][j][k][l] >= 0{
						onnx__Conv_735[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_735[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_735[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_735[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_736 = vec![BN254Fr::default();96]; 
	for i in 0..96 {
		if i_input.onnx__Conv_736[i] >= 0{
			onnx__Conv_736[i] = BN254Fr::from((i_input.onnx__Conv_736[i]) as u64); 
		} else {
			onnx__Conv_736[i] = -BN254Fr::from((-i_input.onnx__Conv_736[i]) as u64); 
		} 
	}
	let mut onnx__Conv_736_q = vec![vec![vec![BN254Fr::default();1];1];96]; 
	for i in 0..96 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_736_q[i][j][k] >= 0{
					onnx__Conv_736_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_736_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_736_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_736_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_735_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_735_nscale >= 0{
		onnx__Conv_735_nscale = BN254Fr::from((i_input.onnx__Conv_735_nscale) as u64); 
	} else {
		onnx__Conv_735_nscale = -BN254Fr::from((-i_input.onnx__Conv_735_nscale) as u64); 
	} 
	let mut onnx__Conv_735_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_735_dscale >= 0{
		onnx__Conv_735_dscale = BN254Fr::from((i_input.onnx__Conv_735_dscale) as u64); 
	} else {
		onnx__Conv_735_dscale = -BN254Fr::from((-i_input.onnx__Conv_735_dscale) as u64); 
	} 
	let mut _features_features_13_Add_output_0_1nscale = BN254Fr::default(); 
	if i_input._features_features_13_Add_output_0_1nscale >= 0{
		_features_features_13_Add_output_0_1nscale = BN254Fr::from((i_input._features_features_13_Add_output_0_1nscale) as u64); 
	} else {
		_features_features_13_Add_output_0_1nscale = -BN254Fr::from((-i_input._features_features_13_Add_output_0_1nscale) as u64); 
	} 
	let mut _features_features_13_Add_output_0_1dscale = BN254Fr::default(); 
	if i_input._features_features_13_Add_output_0_1dscale >= 0{
		_features_features_13_Add_output_0_1dscale = BN254Fr::from((i_input._features_features_13_Add_output_0_1dscale) as u64); 
	} else {
		_features_features_13_Add_output_0_1dscale = -BN254Fr::from((-i_input._features_features_13_Add_output_0_1dscale) as u64); 
	} 
	let mut _features_features_13_Add_output_0_2nscale = BN254Fr::default(); 
	if i_input._features_features_13_Add_output_0_2nscale >= 0{
		_features_features_13_Add_output_0_2nscale = BN254Fr::from((i_input._features_features_13_Add_output_0_2nscale) as u64); 
	} else {
		_features_features_13_Add_output_0_2nscale = -BN254Fr::from((-i_input._features_features_13_Add_output_0_2nscale) as u64); 
	} 
	let mut _features_features_13_Add_output_0_2dscale = BN254Fr::default(); 
	if i_input._features_features_13_Add_output_0_2dscale >= 0{
		_features_features_13_Add_output_0_2dscale = BN254Fr::from((i_input._features_features_13_Add_output_0_2dscale) as u64); 
	} else {
		_features_features_13_Add_output_0_2dscale = -BN254Fr::from((-i_input._features_features_13_Add_output_0_2dscale) as u64); 
	} 
	let mut onnx__Conv_738 = vec![vec![vec![vec![BN254Fr::default();1];1];96];576]; 
	for i in 0..576 {
		for j in 0..96 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_738[i][j][k][l] >= 0{
						onnx__Conv_738[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_738[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_738[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_738[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_739 = vec![BN254Fr::default();576]; 
	for i in 0..576 {
		if i_input.onnx__Conv_739[i] >= 0{
			onnx__Conv_739[i] = BN254Fr::from((i_input.onnx__Conv_739[i]) as u64); 
		} else {
			onnx__Conv_739[i] = -BN254Fr::from((-i_input.onnx__Conv_739[i]) as u64); 
		} 
	}
	let mut onnx__Conv_739_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_739_q[i][j][k] >= 0{
					onnx__Conv_739_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_739_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_739_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_739_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_738_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_738_nscale >= 0{
		onnx__Conv_738_nscale = BN254Fr::from((i_input.onnx__Conv_738_nscale) as u64); 
	} else {
		onnx__Conv_738_nscale = -BN254Fr::from((-i_input.onnx__Conv_738_nscale) as u64); 
	} 
	let mut onnx__Conv_738_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_738_dscale >= 0{
		onnx__Conv_738_dscale = BN254Fr::from((i_input.onnx__Conv_738_dscale) as u64); 
	} else {
		onnx__Conv_738_dscale = -BN254Fr::from((-i_input.onnx__Conv_738_dscale) as u64); 
	} 
	let mut onnx__PRelu_805_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_805_q[i][j][k] >= 0{
					onnx__PRelu_805_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_805_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_805_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_805_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_805_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_805_nscale >= 0{
		onnx__PRelu_805_nscale = BN254Fr::from((i_input.onnx__PRelu_805_nscale) as u64); 
	} else {
		onnx__PRelu_805_nscale = -BN254Fr::from((-i_input.onnx__PRelu_805_nscale) as u64); 
	} 
	let mut onnx__PRelu_805_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_805_dscale >= 0{
		onnx__PRelu_805_dscale = BN254Fr::from((i_input.onnx__PRelu_805_dscale) as u64); 
	} else {
		onnx__PRelu_805_dscale = -BN254Fr::from((-i_input.onnx__PRelu_805_dscale) as u64); 
	} 
	let mut onnx__PRelu_805_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_805_zero[i] >= 0{
			onnx__PRelu_805_zero[i] = BN254Fr::from((i_input.onnx__PRelu_805_zero[i]) as u64); 
		} else {
			onnx__PRelu_805_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_805_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_741 = vec![vec![vec![vec![BN254Fr::default();3];3];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_741[i][j][k][l] >= 0{
						onnx__Conv_741[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_741[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_741[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_741[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_742 = vec![BN254Fr::default();576]; 
	for i in 0..576 {
		if i_input.onnx__Conv_742[i] >= 0{
			onnx__Conv_742[i] = BN254Fr::from((i_input.onnx__Conv_742[i]) as u64); 
		} else {
			onnx__Conv_742[i] = -BN254Fr::from((-i_input.onnx__Conv_742[i]) as u64); 
		} 
	}
	let mut onnx__Conv_742_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_742_q[i][j][k] >= 0{
					onnx__Conv_742_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_742_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_742_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_742_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_741_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_741_nscale >= 0{
		onnx__Conv_741_nscale = BN254Fr::from((i_input.onnx__Conv_741_nscale) as u64); 
	} else {
		onnx__Conv_741_nscale = -BN254Fr::from((-i_input.onnx__Conv_741_nscale) as u64); 
	} 
	let mut onnx__Conv_741_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_741_dscale >= 0{
		onnx__Conv_741_dscale = BN254Fr::from((i_input.onnx__Conv_741_dscale) as u64); 
	} else {
		onnx__Conv_741_dscale = -BN254Fr::from((-i_input.onnx__Conv_741_dscale) as u64); 
	} 
	let mut onnx__PRelu_806_q = vec![vec![vec![BN254Fr::default();1];1];576]; 
	for i in 0..576 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_806_q[i][j][k] >= 0{
					onnx__PRelu_806_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_806_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_806_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_806_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_806_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_806_nscale >= 0{
		onnx__PRelu_806_nscale = BN254Fr::from((i_input.onnx__PRelu_806_nscale) as u64); 
	} else {
		onnx__PRelu_806_nscale = -BN254Fr::from((-i_input.onnx__PRelu_806_nscale) as u64); 
	} 
	let mut onnx__PRelu_806_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_806_dscale >= 0{
		onnx__PRelu_806_dscale = BN254Fr::from((i_input.onnx__PRelu_806_dscale) as u64); 
	} else {
		onnx__PRelu_806_dscale = -BN254Fr::from((-i_input.onnx__PRelu_806_dscale) as u64); 
	} 
	let mut onnx__PRelu_806_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_806_zero[i] >= 0{
			onnx__PRelu_806_zero[i] = BN254Fr::from((i_input.onnx__PRelu_806_zero[i]) as u64); 
		} else {
			onnx__PRelu_806_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_806_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_744 = vec![vec![vec![vec![BN254Fr::default();1];1];576];160]; 
	for i in 0..160 {
		for j in 0..576 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_744[i][j][k][l] >= 0{
						onnx__Conv_744[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_744[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_744[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_744[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_745 = vec![BN254Fr::default();160]; 
	for i in 0..160 {
		if i_input.onnx__Conv_745[i] >= 0{
			onnx__Conv_745[i] = BN254Fr::from((i_input.onnx__Conv_745[i]) as u64); 
		} else {
			onnx__Conv_745[i] = -BN254Fr::from((-i_input.onnx__Conv_745[i]) as u64); 
		} 
	}
	let mut onnx__Conv_745_q = vec![vec![vec![BN254Fr::default();1];1];160]; 
	for i in 0..160 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_745_q[i][j][k] >= 0{
					onnx__Conv_745_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_745_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_745_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_745_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_744_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_744_nscale >= 0{
		onnx__Conv_744_nscale = BN254Fr::from((i_input.onnx__Conv_744_nscale) as u64); 
	} else {
		onnx__Conv_744_nscale = -BN254Fr::from((-i_input.onnx__Conv_744_nscale) as u64); 
	} 
	let mut onnx__Conv_744_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_744_dscale >= 0{
		onnx__Conv_744_dscale = BN254Fr::from((i_input.onnx__Conv_744_dscale) as u64); 
	} else {
		onnx__Conv_744_dscale = -BN254Fr::from((-i_input.onnx__Conv_744_dscale) as u64); 
	} 
	let mut onnx__Conv_747 = vec![vec![vec![vec![BN254Fr::default();1];1];160];960]; 
	for i in 0..960 {
		for j in 0..160 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_747[i][j][k][l] >= 0{
						onnx__Conv_747[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_747[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_747[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_747[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_748 = vec![BN254Fr::default();960]; 
	for i in 0..960 {
		if i_input.onnx__Conv_748[i] >= 0{
			onnx__Conv_748[i] = BN254Fr::from((i_input.onnx__Conv_748[i]) as u64); 
		} else {
			onnx__Conv_748[i] = -BN254Fr::from((-i_input.onnx__Conv_748[i]) as u64); 
		} 
	}
	let mut onnx__Conv_748_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_748_q[i][j][k] >= 0{
					onnx__Conv_748_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_748_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_748_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_748_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_747_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_747_nscale >= 0{
		onnx__Conv_747_nscale = BN254Fr::from((i_input.onnx__Conv_747_nscale) as u64); 
	} else {
		onnx__Conv_747_nscale = -BN254Fr::from((-i_input.onnx__Conv_747_nscale) as u64); 
	} 
	let mut onnx__Conv_747_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_747_dscale >= 0{
		onnx__Conv_747_dscale = BN254Fr::from((i_input.onnx__Conv_747_dscale) as u64); 
	} else {
		onnx__Conv_747_dscale = -BN254Fr::from((-i_input.onnx__Conv_747_dscale) as u64); 
	} 
	let mut onnx__PRelu_807_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_807_q[i][j][k] >= 0{
					onnx__PRelu_807_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_807_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_807_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_807_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_807_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_807_nscale >= 0{
		onnx__PRelu_807_nscale = BN254Fr::from((i_input.onnx__PRelu_807_nscale) as u64); 
	} else {
		onnx__PRelu_807_nscale = -BN254Fr::from((-i_input.onnx__PRelu_807_nscale) as u64); 
	} 
	let mut onnx__PRelu_807_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_807_dscale >= 0{
		onnx__PRelu_807_dscale = BN254Fr::from((i_input.onnx__PRelu_807_dscale) as u64); 
	} else {
		onnx__PRelu_807_dscale = -BN254Fr::from((-i_input.onnx__PRelu_807_dscale) as u64); 
	} 
	let mut onnx__PRelu_807_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_807_zero[i] >= 0{
			onnx__PRelu_807_zero[i] = BN254Fr::from((i_input.onnx__PRelu_807_zero[i]) as u64); 
		} else {
			onnx__PRelu_807_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_807_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_750 = vec![vec![vec![vec![BN254Fr::default();3];3];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_750[i][j][k][l] >= 0{
						onnx__Conv_750[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_750[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_750[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_750[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_751 = vec![BN254Fr::default();960]; 
	for i in 0..960 {
		if i_input.onnx__Conv_751[i] >= 0{
			onnx__Conv_751[i] = BN254Fr::from((i_input.onnx__Conv_751[i]) as u64); 
		} else {
			onnx__Conv_751[i] = -BN254Fr::from((-i_input.onnx__Conv_751[i]) as u64); 
		} 
	}
	let mut onnx__Conv_751_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_751_q[i][j][k] >= 0{
					onnx__Conv_751_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_751_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_751_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_751_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_750_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_750_nscale >= 0{
		onnx__Conv_750_nscale = BN254Fr::from((i_input.onnx__Conv_750_nscale) as u64); 
	} else {
		onnx__Conv_750_nscale = -BN254Fr::from((-i_input.onnx__Conv_750_nscale) as u64); 
	} 
	let mut onnx__Conv_750_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_750_dscale >= 0{
		onnx__Conv_750_dscale = BN254Fr::from((i_input.onnx__Conv_750_dscale) as u64); 
	} else {
		onnx__Conv_750_dscale = -BN254Fr::from((-i_input.onnx__Conv_750_dscale) as u64); 
	} 
	let mut onnx__PRelu_808_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_808_q[i][j][k] >= 0{
					onnx__PRelu_808_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_808_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_808_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_808_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_808_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_808_nscale >= 0{
		onnx__PRelu_808_nscale = BN254Fr::from((i_input.onnx__PRelu_808_nscale) as u64); 
	} else {
		onnx__PRelu_808_nscale = -BN254Fr::from((-i_input.onnx__PRelu_808_nscale) as u64); 
	} 
	let mut onnx__PRelu_808_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_808_dscale >= 0{
		onnx__PRelu_808_dscale = BN254Fr::from((i_input.onnx__PRelu_808_dscale) as u64); 
	} else {
		onnx__PRelu_808_dscale = -BN254Fr::from((-i_input.onnx__PRelu_808_dscale) as u64); 
	} 
	let mut onnx__PRelu_808_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_808_zero[i] >= 0{
			onnx__PRelu_808_zero[i] = BN254Fr::from((i_input.onnx__PRelu_808_zero[i]) as u64); 
		} else {
			onnx__PRelu_808_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_808_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_753 = vec![vec![vec![vec![BN254Fr::default();1];1];960];160]; 
	for i in 0..160 {
		for j in 0..960 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_753[i][j][k][l] >= 0{
						onnx__Conv_753[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_753[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_753[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_753[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_754 = vec![BN254Fr::default();160]; 
	for i in 0..160 {
		if i_input.onnx__Conv_754[i] >= 0{
			onnx__Conv_754[i] = BN254Fr::from((i_input.onnx__Conv_754[i]) as u64); 
		} else {
			onnx__Conv_754[i] = -BN254Fr::from((-i_input.onnx__Conv_754[i]) as u64); 
		} 
	}
	let mut onnx__Conv_754_q = vec![vec![vec![BN254Fr::default();1];1];160]; 
	for i in 0..160 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_754_q[i][j][k] >= 0{
					onnx__Conv_754_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_754_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_754_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_754_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_753_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_753_nscale >= 0{
		onnx__Conv_753_nscale = BN254Fr::from((i_input.onnx__Conv_753_nscale) as u64); 
	} else {
		onnx__Conv_753_nscale = -BN254Fr::from((-i_input.onnx__Conv_753_nscale) as u64); 
	} 
	let mut onnx__Conv_753_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_753_dscale >= 0{
		onnx__Conv_753_dscale = BN254Fr::from((i_input.onnx__Conv_753_dscale) as u64); 
	} else {
		onnx__Conv_753_dscale = -BN254Fr::from((-i_input.onnx__Conv_753_dscale) as u64); 
	} 
	let mut _features_features_15_Add_output_0_1nscale = BN254Fr::default(); 
	if i_input._features_features_15_Add_output_0_1nscale >= 0{
		_features_features_15_Add_output_0_1nscale = BN254Fr::from((i_input._features_features_15_Add_output_0_1nscale) as u64); 
	} else {
		_features_features_15_Add_output_0_1nscale = -BN254Fr::from((-i_input._features_features_15_Add_output_0_1nscale) as u64); 
	} 
	let mut _features_features_15_Add_output_0_1dscale = BN254Fr::default(); 
	if i_input._features_features_15_Add_output_0_1dscale >= 0{
		_features_features_15_Add_output_0_1dscale = BN254Fr::from((i_input._features_features_15_Add_output_0_1dscale) as u64); 
	} else {
		_features_features_15_Add_output_0_1dscale = -BN254Fr::from((-i_input._features_features_15_Add_output_0_1dscale) as u64); 
	} 
	let mut _features_features_15_Add_output_0_2nscale = BN254Fr::default(); 
	if i_input._features_features_15_Add_output_0_2nscale >= 0{
		_features_features_15_Add_output_0_2nscale = BN254Fr::from((i_input._features_features_15_Add_output_0_2nscale) as u64); 
	} else {
		_features_features_15_Add_output_0_2nscale = -BN254Fr::from((-i_input._features_features_15_Add_output_0_2nscale) as u64); 
	} 
	let mut _features_features_15_Add_output_0_2dscale = BN254Fr::default(); 
	if i_input._features_features_15_Add_output_0_2dscale >= 0{
		_features_features_15_Add_output_0_2dscale = BN254Fr::from((i_input._features_features_15_Add_output_0_2dscale) as u64); 
	} else {
		_features_features_15_Add_output_0_2dscale = -BN254Fr::from((-i_input._features_features_15_Add_output_0_2dscale) as u64); 
	} 
	let mut onnx__Conv_756 = vec![vec![vec![vec![BN254Fr::default();1];1];160];960]; 
	for i in 0..960 {
		for j in 0..160 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_756[i][j][k][l] >= 0{
						onnx__Conv_756[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_756[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_756[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_756[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_757 = vec![BN254Fr::default();960]; 
	for i in 0..960 {
		if i_input.onnx__Conv_757[i] >= 0{
			onnx__Conv_757[i] = BN254Fr::from((i_input.onnx__Conv_757[i]) as u64); 
		} else {
			onnx__Conv_757[i] = -BN254Fr::from((-i_input.onnx__Conv_757[i]) as u64); 
		} 
	}
	let mut onnx__Conv_757_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_757_q[i][j][k] >= 0{
					onnx__Conv_757_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_757_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_757_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_757_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_756_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_756_nscale >= 0{
		onnx__Conv_756_nscale = BN254Fr::from((i_input.onnx__Conv_756_nscale) as u64); 
	} else {
		onnx__Conv_756_nscale = -BN254Fr::from((-i_input.onnx__Conv_756_nscale) as u64); 
	} 
	let mut onnx__Conv_756_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_756_dscale >= 0{
		onnx__Conv_756_dscale = BN254Fr::from((i_input.onnx__Conv_756_dscale) as u64); 
	} else {
		onnx__Conv_756_dscale = -BN254Fr::from((-i_input.onnx__Conv_756_dscale) as u64); 
	} 
	let mut onnx__PRelu_809_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_809_q[i][j][k] >= 0{
					onnx__PRelu_809_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_809_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_809_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_809_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_809_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_809_nscale >= 0{
		onnx__PRelu_809_nscale = BN254Fr::from((i_input.onnx__PRelu_809_nscale) as u64); 
	} else {
		onnx__PRelu_809_nscale = -BN254Fr::from((-i_input.onnx__PRelu_809_nscale) as u64); 
	} 
	let mut onnx__PRelu_809_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_809_dscale >= 0{
		onnx__PRelu_809_dscale = BN254Fr::from((i_input.onnx__PRelu_809_dscale) as u64); 
	} else {
		onnx__PRelu_809_dscale = -BN254Fr::from((-i_input.onnx__PRelu_809_dscale) as u64); 
	} 
	let mut onnx__PRelu_809_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_809_zero[i] >= 0{
			onnx__PRelu_809_zero[i] = BN254Fr::from((i_input.onnx__PRelu_809_zero[i]) as u64); 
		} else {
			onnx__PRelu_809_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_809_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_759 = vec![vec![vec![vec![BN254Fr::default();3];3];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_759[i][j][k][l] >= 0{
						onnx__Conv_759[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_759[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_759[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_759[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_760 = vec![BN254Fr::default();960]; 
	for i in 0..960 {
		if i_input.onnx__Conv_760[i] >= 0{
			onnx__Conv_760[i] = BN254Fr::from((i_input.onnx__Conv_760[i]) as u64); 
		} else {
			onnx__Conv_760[i] = -BN254Fr::from((-i_input.onnx__Conv_760[i]) as u64); 
		} 
	}
	let mut onnx__Conv_760_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_760_q[i][j][k] >= 0{
					onnx__Conv_760_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_760_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_760_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_760_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_759_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_759_nscale >= 0{
		onnx__Conv_759_nscale = BN254Fr::from((i_input.onnx__Conv_759_nscale) as u64); 
	} else {
		onnx__Conv_759_nscale = -BN254Fr::from((-i_input.onnx__Conv_759_nscale) as u64); 
	} 
	let mut onnx__Conv_759_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_759_dscale >= 0{
		onnx__Conv_759_dscale = BN254Fr::from((i_input.onnx__Conv_759_dscale) as u64); 
	} else {
		onnx__Conv_759_dscale = -BN254Fr::from((-i_input.onnx__Conv_759_dscale) as u64); 
	} 
	let mut onnx__PRelu_810_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_810_q[i][j][k] >= 0{
					onnx__PRelu_810_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_810_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_810_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_810_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_810_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_810_nscale >= 0{
		onnx__PRelu_810_nscale = BN254Fr::from((i_input.onnx__PRelu_810_nscale) as u64); 
	} else {
		onnx__PRelu_810_nscale = -BN254Fr::from((-i_input.onnx__PRelu_810_nscale) as u64); 
	} 
	let mut onnx__PRelu_810_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_810_dscale >= 0{
		onnx__PRelu_810_dscale = BN254Fr::from((i_input.onnx__PRelu_810_dscale) as u64); 
	} else {
		onnx__PRelu_810_dscale = -BN254Fr::from((-i_input.onnx__PRelu_810_dscale) as u64); 
	} 
	let mut onnx__PRelu_810_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_810_zero[i] >= 0{
			onnx__PRelu_810_zero[i] = BN254Fr::from((i_input.onnx__PRelu_810_zero[i]) as u64); 
		} else {
			onnx__PRelu_810_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_810_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_762 = vec![vec![vec![vec![BN254Fr::default();1];1];960];160]; 
	for i in 0..160 {
		for j in 0..960 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_762[i][j][k][l] >= 0{
						onnx__Conv_762[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_762[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_762[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_762[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_763 = vec![BN254Fr::default();160]; 
	for i in 0..160 {
		if i_input.onnx__Conv_763[i] >= 0{
			onnx__Conv_763[i] = BN254Fr::from((i_input.onnx__Conv_763[i]) as u64); 
		} else {
			onnx__Conv_763[i] = -BN254Fr::from((-i_input.onnx__Conv_763[i]) as u64); 
		} 
	}
	let mut onnx__Conv_763_q = vec![vec![vec![BN254Fr::default();1];1];160]; 
	for i in 0..160 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_763_q[i][j][k] >= 0{
					onnx__Conv_763_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_763_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_763_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_763_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_762_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_762_nscale >= 0{
		onnx__Conv_762_nscale = BN254Fr::from((i_input.onnx__Conv_762_nscale) as u64); 
	} else {
		onnx__Conv_762_nscale = -BN254Fr::from((-i_input.onnx__Conv_762_nscale) as u64); 
	} 
	let mut onnx__Conv_762_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_762_dscale >= 0{
		onnx__Conv_762_dscale = BN254Fr::from((i_input.onnx__Conv_762_dscale) as u64); 
	} else {
		onnx__Conv_762_dscale = -BN254Fr::from((-i_input.onnx__Conv_762_dscale) as u64); 
	} 
	let mut _features_features_16_Add_output_0_1nscale = BN254Fr::default(); 
	if i_input._features_features_16_Add_output_0_1nscale >= 0{
		_features_features_16_Add_output_0_1nscale = BN254Fr::from((i_input._features_features_16_Add_output_0_1nscale) as u64); 
	} else {
		_features_features_16_Add_output_0_1nscale = -BN254Fr::from((-i_input._features_features_16_Add_output_0_1nscale) as u64); 
	} 
	let mut _features_features_16_Add_output_0_1dscale = BN254Fr::default(); 
	if i_input._features_features_16_Add_output_0_1dscale >= 0{
		_features_features_16_Add_output_0_1dscale = BN254Fr::from((i_input._features_features_16_Add_output_0_1dscale) as u64); 
	} else {
		_features_features_16_Add_output_0_1dscale = -BN254Fr::from((-i_input._features_features_16_Add_output_0_1dscale) as u64); 
	} 
	let mut _features_features_16_Add_output_0_2nscale = BN254Fr::default(); 
	if i_input._features_features_16_Add_output_0_2nscale >= 0{
		_features_features_16_Add_output_0_2nscale = BN254Fr::from((i_input._features_features_16_Add_output_0_2nscale) as u64); 
	} else {
		_features_features_16_Add_output_0_2nscale = -BN254Fr::from((-i_input._features_features_16_Add_output_0_2nscale) as u64); 
	} 
	let mut _features_features_16_Add_output_0_2dscale = BN254Fr::default(); 
	if i_input._features_features_16_Add_output_0_2dscale >= 0{
		_features_features_16_Add_output_0_2dscale = BN254Fr::from((i_input._features_features_16_Add_output_0_2dscale) as u64); 
	} else {
		_features_features_16_Add_output_0_2dscale = -BN254Fr::from((-i_input._features_features_16_Add_output_0_2dscale) as u64); 
	} 
	let mut onnx__Conv_765 = vec![vec![vec![vec![BN254Fr::default();1];1];160];960]; 
	for i in 0..960 {
		for j in 0..160 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_765[i][j][k][l] >= 0{
						onnx__Conv_765[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_765[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_765[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_765[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_766 = vec![BN254Fr::default();960]; 
	for i in 0..960 {
		if i_input.onnx__Conv_766[i] >= 0{
			onnx__Conv_766[i] = BN254Fr::from((i_input.onnx__Conv_766[i]) as u64); 
		} else {
			onnx__Conv_766[i] = -BN254Fr::from((-i_input.onnx__Conv_766[i]) as u64); 
		} 
	}
	let mut onnx__Conv_766_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_766_q[i][j][k] >= 0{
					onnx__Conv_766_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_766_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_766_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_766_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_765_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_765_nscale >= 0{
		onnx__Conv_765_nscale = BN254Fr::from((i_input.onnx__Conv_765_nscale) as u64); 
	} else {
		onnx__Conv_765_nscale = -BN254Fr::from((-i_input.onnx__Conv_765_nscale) as u64); 
	} 
	let mut onnx__Conv_765_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_765_dscale >= 0{
		onnx__Conv_765_dscale = BN254Fr::from((i_input.onnx__Conv_765_dscale) as u64); 
	} else {
		onnx__Conv_765_dscale = -BN254Fr::from((-i_input.onnx__Conv_765_dscale) as u64); 
	} 
	let mut onnx__PRelu_811_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_811_q[i][j][k] >= 0{
					onnx__PRelu_811_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_811_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_811_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_811_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_811_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_811_nscale >= 0{
		onnx__PRelu_811_nscale = BN254Fr::from((i_input.onnx__PRelu_811_nscale) as u64); 
	} else {
		onnx__PRelu_811_nscale = -BN254Fr::from((-i_input.onnx__PRelu_811_nscale) as u64); 
	} 
	let mut onnx__PRelu_811_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_811_dscale >= 0{
		onnx__PRelu_811_dscale = BN254Fr::from((i_input.onnx__PRelu_811_dscale) as u64); 
	} else {
		onnx__PRelu_811_dscale = -BN254Fr::from((-i_input.onnx__PRelu_811_dscale) as u64); 
	} 
	let mut onnx__PRelu_811_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_811_zero[i] >= 0{
			onnx__PRelu_811_zero[i] = BN254Fr::from((i_input.onnx__PRelu_811_zero[i]) as u64); 
		} else {
			onnx__PRelu_811_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_811_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_768 = vec![vec![vec![vec![BN254Fr::default();3];3];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.onnx__Conv_768[i][j][k][l] >= 0{
						onnx__Conv_768[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_768[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_768[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_768[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_769 = vec![BN254Fr::default();960]; 
	for i in 0..960 {
		if i_input.onnx__Conv_769[i] >= 0{
			onnx__Conv_769[i] = BN254Fr::from((i_input.onnx__Conv_769[i]) as u64); 
		} else {
			onnx__Conv_769[i] = -BN254Fr::from((-i_input.onnx__Conv_769[i]) as u64); 
		} 
	}
	let mut onnx__Conv_769_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_769_q[i][j][k] >= 0{
					onnx__Conv_769_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_769_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_769_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_769_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_768_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_768_nscale >= 0{
		onnx__Conv_768_nscale = BN254Fr::from((i_input.onnx__Conv_768_nscale) as u64); 
	} else {
		onnx__Conv_768_nscale = -BN254Fr::from((-i_input.onnx__Conv_768_nscale) as u64); 
	} 
	let mut onnx__Conv_768_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_768_dscale >= 0{
		onnx__Conv_768_dscale = BN254Fr::from((i_input.onnx__Conv_768_dscale) as u64); 
	} else {
		onnx__Conv_768_dscale = -BN254Fr::from((-i_input.onnx__Conv_768_dscale) as u64); 
	} 
	let mut onnx__PRelu_812_q = vec![vec![vec![BN254Fr::default();1];1];960]; 
	for i in 0..960 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_812_q[i][j][k] >= 0{
					onnx__PRelu_812_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_812_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_812_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_812_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_812_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_812_nscale >= 0{
		onnx__PRelu_812_nscale = BN254Fr::from((i_input.onnx__PRelu_812_nscale) as u64); 
	} else {
		onnx__PRelu_812_nscale = -BN254Fr::from((-i_input.onnx__PRelu_812_nscale) as u64); 
	} 
	let mut onnx__PRelu_812_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_812_dscale >= 0{
		onnx__PRelu_812_dscale = BN254Fr::from((i_input.onnx__PRelu_812_dscale) as u64); 
	} else {
		onnx__PRelu_812_dscale = -BN254Fr::from((-i_input.onnx__PRelu_812_dscale) as u64); 
	} 
	let mut onnx__PRelu_812_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_812_zero[i] >= 0{
			onnx__PRelu_812_zero[i] = BN254Fr::from((i_input.onnx__PRelu_812_zero[i]) as u64); 
		} else {
			onnx__PRelu_812_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_812_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_771 = vec![vec![vec![vec![BN254Fr::default();1];1];960];320]; 
	for i in 0..320 {
		for j in 0..960 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_771[i][j][k][l] >= 0{
						onnx__Conv_771[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_771[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_771[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_771[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_772 = vec![BN254Fr::default();320]; 
	for i in 0..320 {
		if i_input.onnx__Conv_772[i] >= 0{
			onnx__Conv_772[i] = BN254Fr::from((i_input.onnx__Conv_772[i]) as u64); 
		} else {
			onnx__Conv_772[i] = -BN254Fr::from((-i_input.onnx__Conv_772[i]) as u64); 
		} 
	}
	let mut onnx__Conv_772_q = vec![vec![vec![BN254Fr::default();1];1];320]; 
	for i in 0..320 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_772_q[i][j][k] >= 0{
					onnx__Conv_772_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_772_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_772_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_772_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_771_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_771_nscale >= 0{
		onnx__Conv_771_nscale = BN254Fr::from((i_input.onnx__Conv_771_nscale) as u64); 
	} else {
		onnx__Conv_771_nscale = -BN254Fr::from((-i_input.onnx__Conv_771_nscale) as u64); 
	} 
	let mut onnx__Conv_771_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_771_dscale >= 0{
		onnx__Conv_771_dscale = BN254Fr::from((i_input.onnx__Conv_771_dscale) as u64); 
	} else {
		onnx__Conv_771_dscale = -BN254Fr::from((-i_input.onnx__Conv_771_dscale) as u64); 
	} 
	let mut onnx__Conv_774 = vec![vec![vec![vec![BN254Fr::default();1];1];320];512]; 
	for i in 0..512 {
		for j in 0..320 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input.onnx__Conv_774[i][j][k][l] >= 0{
						onnx__Conv_774[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_774[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_774[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_774[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_775 = vec![BN254Fr::default();512]; 
	for i in 0..512 {
		if i_input.onnx__Conv_775[i] >= 0{
			onnx__Conv_775[i] = BN254Fr::from((i_input.onnx__Conv_775[i]) as u64); 
		} else {
			onnx__Conv_775[i] = -BN254Fr::from((-i_input.onnx__Conv_775[i]) as u64); 
		} 
	}
	let mut onnx__Conv_775_q = vec![vec![vec![BN254Fr::default();1];1];512]; 
	for i in 0..512 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_775_q[i][j][k] >= 0{
					onnx__Conv_775_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_775_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_775_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_775_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_774_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_774_nscale >= 0{
		onnx__Conv_774_nscale = BN254Fr::from((i_input.onnx__Conv_774_nscale) as u64); 
	} else {
		onnx__Conv_774_nscale = -BN254Fr::from((-i_input.onnx__Conv_774_nscale) as u64); 
	} 
	let mut onnx__Conv_774_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_774_dscale >= 0{
		onnx__Conv_774_dscale = BN254Fr::from((i_input.onnx__Conv_774_dscale) as u64); 
	} else {
		onnx__Conv_774_dscale = -BN254Fr::from((-i_input.onnx__Conv_774_dscale) as u64); 
	} 
	let mut onnx__PRelu_813_q = vec![vec![vec![BN254Fr::default();1];1];512]; 
	for i in 0..512 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__PRelu_813_q[i][j][k] >= 0{
					onnx__PRelu_813_q[i][j][k] = BN254Fr::from((i_input.onnx__PRelu_813_q[i][j][k]) as u64); 
				} else {
					onnx__PRelu_813_q[i][j][k] = -BN254Fr::from((-i_input.onnx__PRelu_813_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__PRelu_813_nscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_813_nscale >= 0{
		onnx__PRelu_813_nscale = BN254Fr::from((i_input.onnx__PRelu_813_nscale) as u64); 
	} else {
		onnx__PRelu_813_nscale = -BN254Fr::from((-i_input.onnx__PRelu_813_nscale) as u64); 
	} 
	let mut onnx__PRelu_813_dscale = BN254Fr::default(); 
	if i_input.onnx__PRelu_813_dscale >= 0{
		onnx__PRelu_813_dscale = BN254Fr::from((i_input.onnx__PRelu_813_dscale) as u64); 
	} else {
		onnx__PRelu_813_dscale = -BN254Fr::from((-i_input.onnx__PRelu_813_dscale) as u64); 
	} 
	let mut onnx__PRelu_813_zero = vec![BN254Fr::default();1]; 
	for i in 0..1 {
		if i_input.onnx__PRelu_813_zero[i] >= 0{
			onnx__PRelu_813_zero[i] = BN254Fr::from((i_input.onnx__PRelu_813_zero[i]) as u64); 
		} else {
			onnx__PRelu_813_zero[i] = -BN254Fr::from((-i_input.onnx__PRelu_813_zero[i]) as u64); 
		} 
	}
	let mut onnx__Conv_777 = vec![vec![vec![vec![BN254Fr::default();7];7];1];512]; 
	for i in 0..512 {
		for j in 0..1 {
			for k in 0..7 {
				for l in 0..7 {
					if i_input.onnx__Conv_777[i][j][k][l] >= 0{
						onnx__Conv_777[i][j][k][l] = BN254Fr::from((i_input.onnx__Conv_777[i][j][k][l]) as u64); 
					} else {
						onnx__Conv_777[i][j][k][l] = -BN254Fr::from((-i_input.onnx__Conv_777[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__Conv_778 = vec![BN254Fr::default();512]; 
	for i in 0..512 {
		if i_input.onnx__Conv_778[i] >= 0{
			onnx__Conv_778[i] = BN254Fr::from((i_input.onnx__Conv_778[i]) as u64); 
		} else {
			onnx__Conv_778[i] = -BN254Fr::from((-i_input.onnx__Conv_778[i]) as u64); 
		} 
	}
	let mut onnx__Conv_778_q = vec![vec![vec![BN254Fr::default();1];1];512]; 
	for i in 0..512 {
		for j in 0..1 {
			for k in 0..1 {
				if i_input.onnx__Conv_778_q[i][j][k] >= 0{
					onnx__Conv_778_q[i][j][k] = BN254Fr::from((i_input.onnx__Conv_778_q[i][j][k]) as u64); 
				} else {
					onnx__Conv_778_q[i][j][k] = -BN254Fr::from((-i_input.onnx__Conv_778_q[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut onnx__Conv_777_nscale = BN254Fr::default(); 
	if i_input.onnx__Conv_777_nscale >= 0{
		onnx__Conv_777_nscale = BN254Fr::from((i_input.onnx__Conv_777_nscale) as u64); 
	} else {
		onnx__Conv_777_nscale = -BN254Fr::from((-i_input.onnx__Conv_777_nscale) as u64); 
	} 
	let mut onnx__Conv_777_dscale = BN254Fr::default(); 
	if i_input.onnx__Conv_777_dscale >= 0{
		onnx__Conv_777_dscale = BN254Fr::from((i_input.onnx__Conv_777_dscale) as u64); 
	} else {
		onnx__Conv_777_dscale = -BN254Fr::from((-i_input.onnx__Conv_777_dscale) as u64); 
	} 
	let mut onnx__MatMul_814 = vec![vec![BN254Fr::default();512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			if i_input.onnx__MatMul_814[i][j] >= 0{
				onnx__MatMul_814[i][j] = BN254Fr::from((i_input.onnx__MatMul_814[i][j]) as u64); 
			} else {
				onnx__MatMul_814[i][j] = -BN254Fr::from((-i_input.onnx__MatMul_814[i][j]) as u64); 
			} 
		}
	}
	let mut onnx__MatMul_814_nscale = BN254Fr::default(); 
	if i_input.onnx__MatMul_814_nscale >= 0{
		onnx__MatMul_814_nscale = BN254Fr::from((i_input.onnx__MatMul_814_nscale) as u64); 
	} else {
		onnx__MatMul_814_nscale = -BN254Fr::from((-i_input.onnx__MatMul_814_nscale) as u64); 
	} 
	let mut onnx__MatMul_814_dscale = BN254Fr::default(); 
	if i_input.onnx__MatMul_814_dscale >= 0{
		onnx__MatMul_814_dscale = BN254Fr::from((i_input.onnx__MatMul_814_dscale) as u64); 
	} else {
		onnx__MatMul_814_dscale = -BN254Fr::from((-i_input.onnx__MatMul_814_dscale) as u64); 
	} 
	let ass = Circuit{output,input,_features_features_0_features_0_0_Conv_output_0_conv,_features_features_0_features_0_0_Conv_output_0_floor,_features_features_0_features_0_0_Conv_output_0_relu,_features_features_0_features_0_0_Conv_output_0_min,_features_features_0_features_0_2_PRelu_output_0,_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv,_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor,_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu,_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min,_features_features_1_conv_conv_0_conv_0_2_PRelu_output_0,_features_features_1_conv_conv_1_Conv_output_0_conv,_features_features_1_conv_conv_1_Conv_output_0_floor,_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv,_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor,_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu,_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min,_features_features_2_conv_conv_0_conv_0_2_PRelu_output_0,_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_conv,_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_floor,_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_relu,_features_features_2_conv_conv_1_conv_1_0_Conv_output_0_min,_features_features_2_conv_conv_1_conv_1_2_PRelu_output_0,_features_features_2_conv_conv_2_Conv_output_0_conv,_features_features_2_conv_conv_2_Conv_output_0_floor,_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_conv,_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_floor,_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_relu,_features_features_3_conv_conv_0_conv_0_0_Conv_output_0_min,_features_features_3_conv_conv_0_conv_0_2_PRelu_output_0,onnx__Conv_621,onnx__Conv_622,onnx__Conv_622_q,onnx__Conv_621_nscale,onnx__Conv_621_dscale,onnx__PRelu_779_q,onnx__PRelu_779_nscale,onnx__PRelu_779_dscale,onnx__PRelu_779_zero,onnx__Conv_624,onnx__Conv_625,onnx__Conv_625_q,onnx__Conv_624_nscale,onnx__Conv_624_dscale,onnx__PRelu_780_q,onnx__PRelu_780_nscale,onnx__PRelu_780_dscale,onnx__PRelu_780_zero,onnx__Conv_627,onnx__Conv_628,onnx__Conv_628_q,onnx__Conv_627_nscale,onnx__Conv_627_dscale,onnx__Conv_630,onnx__Conv_631,onnx__Conv_631_q,onnx__Conv_630_nscale,onnx__Conv_630_dscale,onnx__PRelu_781_q,onnx__PRelu_781_nscale,onnx__PRelu_781_dscale,onnx__PRelu_781_zero,onnx__Conv_633,onnx__Conv_634,onnx__Conv_634_q,onnx__Conv_633_nscale,onnx__Conv_633_dscale,onnx__PRelu_782_q,onnx__PRelu_782_nscale,onnx__PRelu_782_dscale,onnx__PRelu_782_zero,onnx__Conv_636,onnx__Conv_637,onnx__Conv_637_q,onnx__Conv_636_nscale,onnx__Conv_636_dscale,onnx__Conv_639,onnx__Conv_640,onnx__Conv_640_q,onnx__Conv_639_nscale,onnx__Conv_639_dscale,onnx__PRelu_783_q,onnx__PRelu_783_nscale,onnx__PRelu_783_dscale,onnx__PRelu_783_zero,onnx__Conv_642,onnx__Conv_643,onnx__Conv_643_q,onnx__Conv_642_nscale,onnx__Conv_642_dscale,onnx__PRelu_784_q,onnx__PRelu_784_nscale,onnx__PRelu_784_dscale,onnx__PRelu_784_zero,onnx__Conv_645,onnx__Conv_646,onnx__Conv_646_q,onnx__Conv_645_nscale,onnx__Conv_645_dscale,_features_features_3_Add_output_0_1nscale,_features_features_3_Add_output_0_1dscale,_features_features_3_Add_output_0_2nscale,_features_features_3_Add_output_0_2dscale,onnx__Conv_648,onnx__Conv_649,onnx__Conv_649_q,onnx__Conv_648_nscale,onnx__Conv_648_dscale,onnx__PRelu_785_q,onnx__PRelu_785_nscale,onnx__PRelu_785_dscale,onnx__PRelu_785_zero,onnx__Conv_651,onnx__Conv_652,onnx__Conv_652_q,onnx__Conv_651_nscale,onnx__Conv_651_dscale,onnx__PRelu_786_q,onnx__PRelu_786_nscale,onnx__PRelu_786_dscale,onnx__PRelu_786_zero,onnx__Conv_654,onnx__Conv_655,onnx__Conv_655_q,onnx__Conv_654_nscale,onnx__Conv_654_dscale,onnx__Conv_657,onnx__Conv_658,onnx__Conv_658_q,onnx__Conv_657_nscale,onnx__Conv_657_dscale,onnx__PRelu_787_q,onnx__PRelu_787_nscale,onnx__PRelu_787_dscale,onnx__PRelu_787_zero,onnx__Conv_660,onnx__Conv_661,onnx__Conv_661_q,onnx__Conv_660_nscale,onnx__Conv_660_dscale,onnx__PRelu_788_q,onnx__PRelu_788_nscale,onnx__PRelu_788_dscale,onnx__PRelu_788_zero,onnx__Conv_663,onnx__Conv_664,onnx__Conv_664_q,onnx__Conv_663_nscale,onnx__Conv_663_dscale,_features_features_5_Add_output_0_1nscale,_features_features_5_Add_output_0_1dscale,_features_features_5_Add_output_0_2nscale,_features_features_5_Add_output_0_2dscale,onnx__Conv_666,onnx__Conv_667,onnx__Conv_667_q,onnx__Conv_666_nscale,onnx__Conv_666_dscale,onnx__PRelu_789_q,onnx__PRelu_789_nscale,onnx__PRelu_789_dscale,onnx__PRelu_789_zero,onnx__Conv_669,onnx__Conv_670,onnx__Conv_670_q,onnx__Conv_669_nscale,onnx__Conv_669_dscale,onnx__PRelu_790_q,onnx__PRelu_790_nscale,onnx__PRelu_790_dscale,onnx__PRelu_790_zero,onnx__Conv_672,onnx__Conv_673,onnx__Conv_673_q,onnx__Conv_672_nscale,onnx__Conv_672_dscale,_features_features_6_Add_output_0_1nscale,_features_features_6_Add_output_0_1dscale,_features_features_6_Add_output_0_2nscale,_features_features_6_Add_output_0_2dscale,onnx__Conv_675,onnx__Conv_676,onnx__Conv_676_q,onnx__Conv_675_nscale,onnx__Conv_675_dscale,onnx__PRelu_791_q,onnx__PRelu_791_nscale,onnx__PRelu_791_dscale,onnx__PRelu_791_zero,onnx__Conv_678,onnx__Conv_679,onnx__Conv_679_q,onnx__Conv_678_nscale,onnx__Conv_678_dscale,onnx__PRelu_792_q,onnx__PRelu_792_nscale,onnx__PRelu_792_dscale,onnx__PRelu_792_zero,onnx__Conv_681,onnx__Conv_682,onnx__Conv_682_q,onnx__Conv_681_nscale,onnx__Conv_681_dscale,onnx__Conv_684,onnx__Conv_685,onnx__Conv_685_q,onnx__Conv_684_nscale,onnx__Conv_684_dscale,onnx__PRelu_793_q,onnx__PRelu_793_nscale,onnx__PRelu_793_dscale,onnx__PRelu_793_zero,onnx__Conv_687,onnx__Conv_688,onnx__Conv_688_q,onnx__Conv_687_nscale,onnx__Conv_687_dscale,onnx__PRelu_794_q,onnx__PRelu_794_nscale,onnx__PRelu_794_dscale,onnx__PRelu_794_zero,onnx__Conv_690,onnx__Conv_691,onnx__Conv_691_q,onnx__Conv_690_nscale,onnx__Conv_690_dscale,_features_features_8_Add_output_0_1nscale,_features_features_8_Add_output_0_1dscale,_features_features_8_Add_output_0_2nscale,_features_features_8_Add_output_0_2dscale,onnx__Conv_693,onnx__Conv_694,onnx__Conv_694_q,onnx__Conv_693_nscale,onnx__Conv_693_dscale,onnx__PRelu_795_q,onnx__PRelu_795_nscale,onnx__PRelu_795_dscale,onnx__PRelu_795_zero,onnx__Conv_696,onnx__Conv_697,onnx__Conv_697_q,onnx__Conv_696_nscale,onnx__Conv_696_dscale,onnx__PRelu_796_q,onnx__PRelu_796_nscale,onnx__PRelu_796_dscale,onnx__PRelu_796_zero,onnx__Conv_699,onnx__Conv_700,onnx__Conv_700_q,onnx__Conv_699_nscale,onnx__Conv_699_dscale,_features_features_9_Add_output_0_1nscale,_features_features_9_Add_output_0_1dscale,_features_features_9_Add_output_0_2nscale,_features_features_9_Add_output_0_2dscale,onnx__Conv_702,onnx__Conv_703,onnx__Conv_703_q,onnx__Conv_702_nscale,onnx__Conv_702_dscale,onnx__PRelu_797_q,onnx__PRelu_797_nscale,onnx__PRelu_797_dscale,onnx__PRelu_797_zero,onnx__Conv_705,onnx__Conv_706,onnx__Conv_706_q,onnx__Conv_705_nscale,onnx__Conv_705_dscale,onnx__PRelu_798_q,onnx__PRelu_798_nscale,onnx__PRelu_798_dscale,onnx__PRelu_798_zero,onnx__Conv_708,onnx__Conv_709,onnx__Conv_709_q,onnx__Conv_708_nscale,onnx__Conv_708_dscale,_features_features_10_Add_output_0_1nscale,_features_features_10_Add_output_0_1dscale,_features_features_10_Add_output_0_2nscale,_features_features_10_Add_output_0_2dscale,onnx__Conv_711,onnx__Conv_712,onnx__Conv_712_q,onnx__Conv_711_nscale,onnx__Conv_711_dscale,onnx__PRelu_799_q,onnx__PRelu_799_nscale,onnx__PRelu_799_dscale,onnx__PRelu_799_zero,onnx__Conv_714,onnx__Conv_715,onnx__Conv_715_q,onnx__Conv_714_nscale,onnx__Conv_714_dscale,onnx__PRelu_800_q,onnx__PRelu_800_nscale,onnx__PRelu_800_dscale,onnx__PRelu_800_zero,onnx__Conv_717,onnx__Conv_718,onnx__Conv_718_q,onnx__Conv_717_nscale,onnx__Conv_717_dscale,onnx__Conv_720,onnx__Conv_721,onnx__Conv_721_q,onnx__Conv_720_nscale,onnx__Conv_720_dscale,onnx__PRelu_801_q,onnx__PRelu_801_nscale,onnx__PRelu_801_dscale,onnx__PRelu_801_zero,onnx__Conv_723,onnx__Conv_724,onnx__Conv_724_q,onnx__Conv_723_nscale,onnx__Conv_723_dscale,onnx__PRelu_802_q,onnx__PRelu_802_nscale,onnx__PRelu_802_dscale,onnx__PRelu_802_zero,onnx__Conv_726,onnx__Conv_727,onnx__Conv_727_q,onnx__Conv_726_nscale,onnx__Conv_726_dscale,_features_features_12_Add_output_0_1nscale,_features_features_12_Add_output_0_1dscale,_features_features_12_Add_output_0_2nscale,_features_features_12_Add_output_0_2dscale,onnx__Conv_729,onnx__Conv_730,onnx__Conv_730_q,onnx__Conv_729_nscale,onnx__Conv_729_dscale,onnx__PRelu_803_q,onnx__PRelu_803_nscale,onnx__PRelu_803_dscale,onnx__PRelu_803_zero,onnx__Conv_732,onnx__Conv_733,onnx__Conv_733_q,onnx__Conv_732_nscale,onnx__Conv_732_dscale,onnx__PRelu_804_q,onnx__PRelu_804_nscale,onnx__PRelu_804_dscale,onnx__PRelu_804_zero,onnx__Conv_735,onnx__Conv_736,onnx__Conv_736_q,onnx__Conv_735_nscale,onnx__Conv_735_dscale,_features_features_13_Add_output_0_1nscale,_features_features_13_Add_output_0_1dscale,_features_features_13_Add_output_0_2nscale,_features_features_13_Add_output_0_2dscale,onnx__Conv_738,onnx__Conv_739,onnx__Conv_739_q,onnx__Conv_738_nscale,onnx__Conv_738_dscale,onnx__PRelu_805_q,onnx__PRelu_805_nscale,onnx__PRelu_805_dscale,onnx__PRelu_805_zero,onnx__Conv_741,onnx__Conv_742,onnx__Conv_742_q,onnx__Conv_741_nscale,onnx__Conv_741_dscale,onnx__PRelu_806_q,onnx__PRelu_806_nscale,onnx__PRelu_806_dscale,onnx__PRelu_806_zero,onnx__Conv_744,onnx__Conv_745,onnx__Conv_745_q,onnx__Conv_744_nscale,onnx__Conv_744_dscale,onnx__Conv_747,onnx__Conv_748,onnx__Conv_748_q,onnx__Conv_747_nscale,onnx__Conv_747_dscale,onnx__PRelu_807_q,onnx__PRelu_807_nscale,onnx__PRelu_807_dscale,onnx__PRelu_807_zero,onnx__Conv_750,onnx__Conv_751,onnx__Conv_751_q,onnx__Conv_750_nscale,onnx__Conv_750_dscale,onnx__PRelu_808_q,onnx__PRelu_808_nscale,onnx__PRelu_808_dscale,onnx__PRelu_808_zero,onnx__Conv_753,onnx__Conv_754,onnx__Conv_754_q,onnx__Conv_753_nscale,onnx__Conv_753_dscale,_features_features_15_Add_output_0_1nscale,_features_features_15_Add_output_0_1dscale,_features_features_15_Add_output_0_2nscale,_features_features_15_Add_output_0_2dscale,onnx__Conv_756,onnx__Conv_757,onnx__Conv_757_q,onnx__Conv_756_nscale,onnx__Conv_756_dscale,onnx__PRelu_809_q,onnx__PRelu_809_nscale,onnx__PRelu_809_dscale,onnx__PRelu_809_zero,onnx__Conv_759,onnx__Conv_760,onnx__Conv_760_q,onnx__Conv_759_nscale,onnx__Conv_759_dscale,onnx__PRelu_810_q,onnx__PRelu_810_nscale,onnx__PRelu_810_dscale,onnx__PRelu_810_zero,onnx__Conv_762,onnx__Conv_763,onnx__Conv_763_q,onnx__Conv_762_nscale,onnx__Conv_762_dscale,_features_features_16_Add_output_0_1nscale,_features_features_16_Add_output_0_1dscale,_features_features_16_Add_output_0_2nscale,_features_features_16_Add_output_0_2dscale,onnx__Conv_765,onnx__Conv_766,onnx__Conv_766_q,onnx__Conv_765_nscale,onnx__Conv_765_dscale,onnx__PRelu_811_q,onnx__PRelu_811_nscale,onnx__PRelu_811_dscale,onnx__PRelu_811_zero,onnx__Conv_768,onnx__Conv_769,onnx__Conv_769_q,onnx__Conv_768_nscale,onnx__Conv_768_dscale,onnx__PRelu_812_q,onnx__PRelu_812_nscale,onnx__PRelu_812_dscale,onnx__PRelu_812_zero,onnx__Conv_771,onnx__Conv_772,onnx__Conv_772_q,onnx__Conv_771_nscale,onnx__Conv_771_dscale,onnx__Conv_774,onnx__Conv_775,onnx__Conv_775_q,onnx__Conv_774_nscale,onnx__Conv_774_dscale,onnx__PRelu_813_q,onnx__PRelu_813_nscale,onnx__PRelu_813_dscale,onnx__PRelu_813_zero,onnx__Conv_777,onnx__Conv_778,onnx__Conv_778_q,onnx__Conv_777_nscale,onnx__Conv_777_dscale,onnx__MatMul_814,onnx__MatMul_814_nscale,onnx__MatMul_814_dscale};
	ass
}

#[test]
fn expander_prover() -> std::io::Result<()>{ 
	let compile_result = stacker::grow(32 * 1024 * 1024 * 1024, ||
		{
			let mut ctx: Context<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>> = Context::default();
			let input_str = fs::read_to_string("input.json").unwrap();
			let input: Circuit_Input = serde_json::from_str(&input_str).unwrap();
			let mut assignment = input_copy(&input);
			let file = std::fs::File::open("circuit.txt").unwrap();
			let reader = std::io::BufReader::new(file);
			let kernels = Vec::<Kernel<BN254Config>>::deserialize_from(reader).unwrap();			// mul operation
			let kernel__features_features_0_features_0_0_Conv_mul = &kernels[0];
			let _features_features_0_features_0_0_Conv_output_0_conv = ctx.copy_to_device(&assignment._features_features_0_features_0_0_Conv_output_0_conv, false);
			let _features_features_0_features_0_0_Conv_output_0_conv_clone = _features_features_0_features_0_0_Conv_output_0_conv.clone();
			let onnx__Conv_621_nscale = ctx.copy_to_device(&assignment.onnx__Conv_621_nscale, true);
			let onnx__Conv_621_nscale_clone = onnx__Conv_621_nscale.clone();
			let mut _features_features_0_features_0_0_Conv_output_0_mul: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_features_0_0_Conv_mul, _features_features_0_features_0_0_Conv_output_0_conv_clone, onnx__Conv_621_nscale_clone, mut _features_features_0_features_0_0_Conv_output_0_mul);
			// div operation
			// add operation
			let kernel__features_features_0_features_0_0_Conv = &kernels[1];
			let _features_features_0_features_0_0_Conv_output_0_floor = ctx.copy_to_device(&assignment._features_features_0_features_0_0_Conv_output_0_floor, false);
			let _features_features_0_features_0_0_Conv_output_0_floor_clone = _features_features_0_features_0_0_Conv_output_0_floor.clone();
			let onnx__Conv_622_q = ctx.copy_to_device(&assignment.onnx__Conv_622_q, true);
			let onnx__Conv_622_q_clone = onnx__Conv_622_q.clone();
			let mut _features_features_0_features_0_0_Conv_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_features_0_0_Conv, _features_features_0_features_0_0_Conv_output_0_floor_clone, onnx__Conv_622_q_clone, mut _features_features_0_features_0_0_Conv_output_0);
			// relu operation
			let kernel__features_features_0_features_0_2_PRelu_relu = &kernels[2];
			let _features_features_0_features_0_0_Conv_output_0_clone = _features_features_0_features_0_0_Conv_output_0.clone();
			let _features_features_0_features_0_0_Conv_output_0_relu = ctx.copy_to_device(&assignment._features_features_0_features_0_0_Conv_output_0_relu, false);
			let _features_features_0_features_0_0_Conv_output_0_relu_clone = _features_features_0_features_0_0_Conv_output_0_relu.clone();
			call_kernel!(ctx, kernel__features_features_0_features_0_2_PRelu_relu, _features_features_0_features_0_0_Conv_output_0_clone, _features_features_0_features_0_0_Conv_output_0_relu_clone);
			// mul operation
			let kernel__features_features_0_features_0_2_PRelu_pos = &kernels[3];
			let _features_features_0_features_0_0_Conv_output_0_relu = ctx.copy_to_device(&assignment._features_features_0_features_0_0_Conv_output_0_relu, false);
			let _features_features_0_features_0_0_Conv_output_0_relu_clone = _features_features_0_features_0_0_Conv_output_0_relu.clone();
			let onnx__PRelu_779_dscale = ctx.copy_to_device(&assignment.onnx__PRelu_779_dscale, true);
			let onnx__PRelu_779_dscale_clone = onnx__PRelu_779_dscale.clone();
			let mut _features_features_0_features_0_0_Conv_output_0_pos: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_features_0_2_PRelu_pos, _features_features_0_features_0_0_Conv_output_0_relu_clone, onnx__PRelu_779_dscale_clone, mut _features_features_0_features_0_0_Conv_output_0_pos);
			// mul operation
			let kernel__features_features_0_features_0_2_PRelu_qn = &kernels[4];
			let onnx__PRelu_779_q = ctx.copy_to_device(&assignment.onnx__PRelu_779_q, false);
			let onnx__PRelu_779_q_clone = onnx__PRelu_779_q.clone();
			let onnx__PRelu_779_nscale = ctx.copy_to_device(&assignment.onnx__PRelu_779_nscale, true);
			let onnx__PRelu_779_nscale_clone = onnx__PRelu_779_nscale.clone();
			let mut onnx__PRelu_779_qn: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_features_0_2_PRelu_qn, onnx__PRelu_779_q_clone, onnx__PRelu_779_nscale_clone, mut onnx__PRelu_779_qn);
			// mul operation
			let kernel__features_features_0_features_0_2_PRelu_neg = &kernels[5];
			let onnx__PRelu_779_qn_clone = onnx__PRelu_779_qn.clone();
			let _features_features_0_features_0_0_Conv_output_0_min = ctx.copy_to_device(&assignment._features_features_0_features_0_0_Conv_output_0_min, false);
			let _features_features_0_features_0_0_Conv_output_0_min_clone = _features_features_0_features_0_0_Conv_output_0_min.clone();
			let mut _features_features_0_features_0_0_Conv_output_0_neg: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_features_0_2_PRelu_neg, onnx__PRelu_779_qn_clone, _features_features_0_features_0_0_Conv_output_0_min_clone, mut _features_features_0_features_0_0_Conv_output_0_neg);
			// add operation
			let kernel__features_features_0_features_0_2_PRelu_prelu = &kernels[6];
			let _features_features_0_features_0_0_Conv_output_0_pos_clone = _features_features_0_features_0_0_Conv_output_0_pos.clone();
			let _features_features_0_features_0_0_Conv_output_0_neg_clone = _features_features_0_features_0_0_Conv_output_0_neg.clone();
			let mut _features_features_0_features_0_0_Conv_output_0_prelu: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_features_0_2_PRelu_prelu, _features_features_0_features_0_0_Conv_output_0_pos_clone, _features_features_0_features_0_0_Conv_output_0_neg_clone, mut _features_features_0_features_0_0_Conv_output_0_prelu);
			// div operation
			// mul operation
			let kernel__features_features_1_conv_conv_0_conv_0_0_Conv_mul = &kernels[7];
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv = ctx.copy_to_device(&assignment._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv, false);
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv_clone = _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv.clone();
			let onnx__Conv_624_nscale = ctx.copy_to_device(&assignment.onnx__Conv_624_nscale, true);
			let onnx__Conv_624_nscale_clone = onnx__Conv_624_nscale.clone();
			let mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_mul: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_1_conv_conv_0_conv_0_0_Conv_mul, _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv_clone, onnx__Conv_624_nscale_clone, mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_mul);
			// div operation
			// add operation
			let kernel__features_features_1_conv_conv_0_conv_0_0_Conv = &kernels[8];
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor = ctx.copy_to_device(&assignment._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor, false);
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor_clone = _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor.clone();
			let onnx__Conv_625_q = ctx.copy_to_device(&assignment.onnx__Conv_625_q, true);
			let onnx__Conv_625_q_clone = onnx__Conv_625_q.clone();
			let mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_1_conv_conv_0_conv_0_0_Conv, _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor_clone, onnx__Conv_625_q_clone, mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0);
			// relu operation
			let kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_relu = &kernels[9];
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_clone = _features_features_1_conv_conv_0_conv_0_0_Conv_output_0.clone();
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu = ctx.copy_to_device(&assignment._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu, false);
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu_clone = _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu.clone();
			call_kernel!(ctx, kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_relu, _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_clone, _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu_clone);
			// mul operation
			let kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_pos = &kernels[10];
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu = ctx.copy_to_device(&assignment._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu, false);
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu_clone = _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu.clone();
			let onnx__PRelu_780_dscale = ctx.copy_to_device(&assignment.onnx__PRelu_780_dscale, true);
			let onnx__PRelu_780_dscale_clone = onnx__PRelu_780_dscale.clone();
			let mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_pos: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_pos, _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu_clone, onnx__PRelu_780_dscale_clone, mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_pos);
			// mul operation
			let kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_qn = &kernels[11];
			let onnx__PRelu_780_q = ctx.copy_to_device(&assignment.onnx__PRelu_780_q, false);
			let onnx__PRelu_780_q_clone = onnx__PRelu_780_q.clone();
			let onnx__PRelu_780_nscale = ctx.copy_to_device(&assignment.onnx__PRelu_780_nscale, true);
			let onnx__PRelu_780_nscale_clone = onnx__PRelu_780_nscale.clone();
			let mut onnx__PRelu_780_qn: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_qn, onnx__PRelu_780_q_clone, onnx__PRelu_780_nscale_clone, mut onnx__PRelu_780_qn);
			// mul operation
			let kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_neg = &kernels[12];
			let onnx__PRelu_780_qn_clone = onnx__PRelu_780_qn.clone();
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min = ctx.copy_to_device(&assignment._features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min, false);
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min_clone = _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min.clone();
			let mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_neg: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_neg, onnx__PRelu_780_qn_clone, _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min_clone, mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_neg);
			// add operation
			let kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_prelu = &kernels[13];
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_pos_clone = _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_pos.clone();
			let _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_neg_clone = _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_neg.clone();
			let mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_prelu: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_prelu, _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_pos_clone, _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_neg_clone, mut _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_prelu);
			// div operation
			// mul operation
			let kernel__features_features_1_conv_conv_1_Conv_mul = &kernels[14];
			let _features_features_1_conv_conv_1_Conv_output_0_conv = ctx.copy_to_device(&assignment._features_features_1_conv_conv_1_Conv_output_0_conv, false);
			let _features_features_1_conv_conv_1_Conv_output_0_conv_clone = _features_features_1_conv_conv_1_Conv_output_0_conv.clone();
			let onnx__Conv_627_nscale = ctx.copy_to_device(&assignment.onnx__Conv_627_nscale, true);
			let onnx__Conv_627_nscale_clone = onnx__Conv_627_nscale.clone();
			let mut _features_features_1_conv_conv_1_Conv_output_0_mul: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_1_conv_conv_1_Conv_mul, _features_features_1_conv_conv_1_Conv_output_0_conv_clone, onnx__Conv_627_nscale_clone, mut _features_features_1_conv_conv_1_Conv_output_0_mul);
			// div operation
			// add operation
			let kernel__features_features_1_conv_conv_1_Conv = &kernels[15];
			let _features_features_1_conv_conv_1_Conv_output_0_floor = ctx.copy_to_device(&assignment._features_features_1_conv_conv_1_Conv_output_0_floor, false);
			let _features_features_1_conv_conv_1_Conv_output_0_floor_clone = _features_features_1_conv_conv_1_Conv_output_0_floor.clone();
			let onnx__Conv_628_q = ctx.copy_to_device(&assignment.onnx__Conv_628_q, true);
			let onnx__Conv_628_q_clone = onnx__Conv_628_q.clone();
			let mut _features_features_1_conv_conv_1_Conv_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_1_conv_conv_1_Conv, _features_features_1_conv_conv_1_Conv_output_0_floor_clone, onnx__Conv_628_q_clone, mut _features_features_1_conv_conv_1_Conv_output_0);
			// mul operation
			let kernel__features_features_2_conv_conv_0_conv_0_0_Conv_mul = &kernels[16];
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv = ctx.copy_to_device(&assignment._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv, false);
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv_clone = _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv.clone();
			let onnx__Conv_630_nscale = ctx.copy_to_device(&assignment.onnx__Conv_630_nscale, true);
			let onnx__Conv_630_nscale_clone = onnx__Conv_630_nscale.clone();
			let mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_mul: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_2_conv_conv_0_conv_0_0_Conv_mul, _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv_clone, onnx__Conv_630_nscale_clone, mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_mul);
			// div operation
			// add operation
			let kernel__features_features_2_conv_conv_0_conv_0_0_Conv = &kernels[17];
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor = ctx.copy_to_device(&assignment._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor, false);
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor_clone = _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor.clone();
			let onnx__Conv_631_q = ctx.copy_to_device(&assignment.onnx__Conv_631_q, true);
			let onnx__Conv_631_q_clone = onnx__Conv_631_q.clone();
			let mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_2_conv_conv_0_conv_0_0_Conv, _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor_clone, onnx__Conv_631_q_clone, mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0);
			// relu operation
			let kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_relu = &kernels[18];
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_clone = _features_features_2_conv_conv_0_conv_0_0_Conv_output_0.clone();
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu = ctx.copy_to_device(&assignment._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu, false);
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu_clone = _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu.clone();
			call_kernel!(ctx, kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_relu, _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_clone, _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu_clone);
			// mul operation
			let kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_pos = &kernels[19];
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu = ctx.copy_to_device(&assignment._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu, false);
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu_clone = _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu.clone();
			let onnx__PRelu_781_dscale = ctx.copy_to_device(&assignment.onnx__PRelu_781_dscale, true);
			let onnx__PRelu_781_dscale_clone = onnx__PRelu_781_dscale.clone();
			let mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_pos: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_pos, _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu_clone, onnx__PRelu_781_dscale_clone, mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_pos);
			// mul operation
			let kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_qn = &kernels[20];
			let onnx__PRelu_781_q = ctx.copy_to_device(&assignment.onnx__PRelu_781_q, false);
			let onnx__PRelu_781_q_clone = onnx__PRelu_781_q.clone();
			let onnx__PRelu_781_nscale = ctx.copy_to_device(&assignment.onnx__PRelu_781_nscale, true);
			let onnx__PRelu_781_nscale_clone = onnx__PRelu_781_nscale.clone();
			let mut onnx__PRelu_781_qn: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_qn, onnx__PRelu_781_q_clone, onnx__PRelu_781_nscale_clone, mut onnx__PRelu_781_qn);
			// mul operation
			let kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_neg = &kernels[21];
			let onnx__PRelu_781_qn_clone = onnx__PRelu_781_qn.clone();
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min = ctx.copy_to_device(&assignment._features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min, false);
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min_clone = _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min.clone();
			let mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_neg: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_neg, onnx__PRelu_781_qn_clone, _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min_clone, mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_neg);
			// add operation
			let kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_prelu = &kernels[22];
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_pos_clone = _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_pos.clone();
			let _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_neg_clone = _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_neg.clone();
			let mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_prelu: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_prelu, _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_pos_clone, _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_neg_clone, mut _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_prelu);
			// div operation
			let computation_graph = ctx.to_computation_graph();
			let file = std::fs::File::create("graph.txt").unwrap();
			let writer = std::io::BufWriter::new(file);
			computation_graph.serialize_into(writer);
			let (prover_setup, _) = ctx.proving_system_setup(&computation_graph);
			let proof = ctx.to_proof(&prover_setup);
			let file = std::fs::File::create("proof.txt").unwrap();
			let writer = std::io::BufWriter::new(file);
			proof.serialize_into(writer);
			<ParallelizedExpanderGKRProvingSystem::<BN254ConfigSha2Hyrax> as ProvingSystem<BN254Config>>::post_process();
		}
	);
	Ok(())
}
