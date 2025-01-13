use extra::*;
use expander_compiler::frontend::*;
use sha2::{Digest, Sha256};
use crate::big_int::{to_binary_hint, big_endian_m31_array_put_uint32, bytes_to_bits, bit_array_to_m31, big_array_add, sigma0, sigma1, cap_sigma0, cap_sigma1, ch, maj, m31_to_bit_array};

const SHA256LEN: usize = 32;
const CHUNK: usize = 64;
const INIT0: u32 = 0x6A09E667;
const INIT1: u32 = 0xBB67AE85;
const INIT2: u32 = 0x3C6EF372;
const INIT3: u32 = 0xA54FF53A;
const INIT4: u32 = 0x510E527F;
const INIT5: u32 = 0x9B05688C;
const INIT6: u32 = 0x1F83D9AB;
const INIT7: u32 = 0x5BE0CD19;
//for m31 field (2^31-1), split each one to 2 30-bit element 
const INIT00: u32 = INIT0 & 0x3FFFFFFF;
const INIT01: u32 = INIT0 >> 30;
const INIT10: u32 = INIT1 & 0x3FFFFFFF;
const INIT11: u32 = INIT1 >> 30;
const INIT20: u32 = INIT2 & 0x3FFFFFFF;
const INIT21: u32 = INIT2 >> 30;
const INIT30: u32 = INIT3 & 0x3FFFFFFF;
const INIT31: u32 = INIT3 >> 30;
const INIT40: u32 = INIT4 & 0x3FFFFFFF;
const INIT41: u32 = INIT4 >> 30;
const INIT50: u32 = INIT5 & 0x3FFFFFFF;
const INIT51: u32 = INIT5 >> 30;
const INIT60: u32 = INIT6 & 0x3FFFFFFF;
const INIT61: u32 = INIT6 >> 30;
const INIT70: u32 = INIT7 & 0x3FFFFFFF;
const INIT71: u32 = INIT7 >> 30;
const _K:[u32;64] = [
	0x428a2f98,
	0x71374491,
	0xb5c0fbcf,
	0xe9b5dba5,
	0x3956c25b,
	0x59f111f1,
	0x923f82a4,
	0xab1c5ed5,
	0xd807aa98,
	0x12835b01,
	0x243185be,
	0x550c7dc3,
	0x72be5d74,
	0x80deb1fe,
	0x9bdc06a7,
	0xc19bf174,
	0xe49b69c1,
	0xefbe4786,
	0x0fc19dc6,
	0x240ca1cc,
	0x2de92c6f,
	0x4a7484aa,
	0x5cb0a9dc,
	0x76f988da,
	0x983e5152,
	0xa831c66d,
	0xb00327c8,
	0xbf597fc7,
	0xc6e00bf3,
	0xd5a79147,
	0x06ca6351,
	0x14292967,
	0x27b70a85,
	0x2e1b2138,
	0x4d2c6dfc,
	0x53380d13,
	0x650a7354,
	0x766a0abb,
	0x81c2c92e,
	0x92722c85,
	0xa2bfe8a1,
	0xa81a664b,
	0xc24b8b70,
	0xc76c51a3,
	0xd192e819,
	0xd6990624,
	0xf40e3585,
	0x106aa070,
	0x19a4c116,
	0x1e376c08,
	0x2748774c,
	0x34b0bcb5,
	0x391c0cb3,
	0x4ed8aa4a,
	0x5b9cca4f,
	0x682e6ff3,
	0x748f82ee,
	0x78a5636f,
	0x84c87814,
	0x8cc70208,
	0x90befffa,
	0xa4506ceb,
	0xbef9a3f7,
	0xc67178f2,
];	
struct MyDigest {
	h: [[Variable; 2]; 8],
	nx: usize,
	len: u64,
	kbits: [[Variable;32]; 64],
}
impl MyDigest {
	fn new<C: Config, B: RootAPI<C>>(api: &mut B) -> Self {
		let mut h = [[api.constant(0); 2]; 8];
		h[0][0] = api.constant(INIT00);
		h[0][1] = api.constant(INIT01);
		h[1][0] = api.constant(INIT10);
		h[1][1] = api.constant(INIT11);
		h[2][0] = api.constant(INIT20);
		h[2][1] = api.constant(INIT21);
		h[3][0] = api.constant(INIT30);
		h[3][1] = api.constant(INIT31);
		h[4][0] = api.constant(INIT40);
		h[4][1] = api.constant(INIT41);
		h[5][0] = api.constant(INIT50);
		h[5][1] = api.constant(INIT51);
		h[6][0] = api.constant(INIT60);
		h[6][1] = api.constant(INIT61);
		h[7][0] = api.constant(INIT70);
		h[7][1] = api.constant(INIT71);
		let mut kbits_u8 = [[0;32];64];
		for i in 0..64 {
			for j in 0..32 {
				kbits_u8[i][j] = ((_K[i] >> j) & 1) as u8;
			}
		}
		let mut kbits = [[api.constant(0); 32]; 64];
		for i in 0..64 {
			for j in 0..32 {
				kbits[i][j] = api.constant(kbits_u8[i][j] as u32);
			}
		}
		MyDigest {
			h,
			nx: 0,
			len: 0,
			kbits,
		}
	}
	fn reset<C: Config, B: RootAPI<C>>(&mut self, api: &mut B) {
		for i in 0..8 {
			self.h[i] = [api.constant(0); 2];
		}
		self.h[0][0] = api.constant(INIT00);
		self.h[0][1] = api.constant(INIT01);
		self.h[1][0] = api.constant(INIT10);
		self.h[1][1] = api.constant(INIT11);
		self.h[2][0] = api.constant(INIT20);
		self.h[2][1] = api.constant(INIT21);
		self.h[3][0] = api.constant(INIT30);
		self.h[3][1] = api.constant(INIT31);
		self.h[4][0] = api.constant(INIT40);
		self.h[4][1] = api.constant(INIT41);
		self.h[5][0] = api.constant(INIT50);
		self.h[5][1] = api.constant(INIT51);
		self.h[6][0] = api.constant(INIT60);
		self.h[6][1] = api.constant(INIT61);
		self.h[7][0] = api.constant(INIT70);
		self.h[7][1] = api.constant(INIT71);
		self.nx = 0;
		self.len = 0;
	}
	//always write a chunk
	fn chunk_write<C: Config, B: RootAPI<C>>(&mut self, api: &mut B, p: &[Variable]) {
		if p.len() != CHUNK || self.nx != 0 {
			panic!("p.len() != CHUNK || self.nx != 0");
		}
		self.len += CHUNK as u64;
		let tmp_h = self.h.clone();
		self.h = self.block(api, tmp_h, p);
	}
	fn return_sum<C: Config, B: RootAPI<C>>(&mut self, api: &mut B) -> [Variable;SHA256LEN] {

		let mut digest = [api.constant(0); SHA256LEN];

		big_endian_m31_array_put_uint32(api, &mut digest[0..], self.h[0]);
		big_endian_m31_array_put_uint32(api, &mut digest[4..], self.h[1]);
		big_endian_m31_array_put_uint32(api, &mut digest[8..], self.h[2]);
		big_endian_m31_array_put_uint32(api, &mut digest[12..], self.h[3]);
		big_endian_m31_array_put_uint32(api, &mut digest[16..], self.h[4]);
		big_endian_m31_array_put_uint32(api, &mut digest[20..], self.h[5]);
		big_endian_m31_array_put_uint32(api, &mut digest[24..], self.h[6]);
		big_endian_m31_array_put_uint32(api, &mut digest[28..], self.h[7]);
		digest
	}
	fn block<C: Config, B: RootAPI<C>>(&mut self, api: &mut B, h: [[Variable;2];8], p: &[Variable]) -> [[Variable;2];8] {
	let mut p = p;
	let mut hh = h;
	while p.len() >= CHUNK {
		let mut msg_schedule = vec![];
		for t in 0..64 {
			if t <= 15 {
				msg_schedule.push(bytes_to_bits(api, &p[t*4..t*4+4]));
			} else {
				let term1_tmp = sigma1(api, &msg_schedule[t-2]);
				let term1 = bit_array_to_m31(api, &term1_tmp);
				let term2 = bit_array_to_m31(api, &msg_schedule[t-7]);
				let term3_tmp = sigma0(api, &msg_schedule[t-15]);
				let term3 = bit_array_to_m31(api, &term3_tmp);
				let term4 = bit_array_to_m31(api, &msg_schedule[t-16]);
				let schedule_tmp1 = big_array_add(api, &term1, &term2, 30);
				let schedule_tmp2 = big_array_add(api, &term3, &term4, 30);
				let schedule = big_array_add(api, &schedule_tmp1, &schedule_tmp2, 30);
				let schedule_bits = m31_to_bit_array(api, &schedule)[..32].to_vec();
				msg_schedule.push(schedule_bits);
			}
		}
		let mut a = hh[0].to_vec();
		let mut b = hh[1].to_vec();
		let mut c = hh[2].to_vec();
		let mut d = hh[3].to_vec();
		let mut e = hh[4].to_vec();
		let mut f = hh[5].to_vec();
		let mut g = hh[6].to_vec();
		let mut h = hh[7].to_vec();

		//rewrite
		let mut a_bit = m31_to_bit_array(api, &a)[..32].to_vec();
		let mut b_bit = m31_to_bit_array(api, &b)[..32].to_vec();
		let mut c_bit = m31_to_bit_array(api, &c)[..32].to_vec();
		let mut e_bit = m31_to_bit_array(api, &e)[..32].to_vec();
		let mut f_bit = m31_to_bit_array(api, &f)[..32].to_vec();
		let mut g_bit = m31_to_bit_array(api, &g)[..32].to_vec();
		for t in 0..64 {
			let mut t1_term1 = [api.constant(0); 2];
			t1_term1[0] = h[0];
			t1_term1[1] = h[1];
			let t1_term2_tmp = cap_sigma1(api, &e_bit);
			let t1_term2 = bit_array_to_m31(api, &t1_term2_tmp);
			let t1_term3_tmp = ch(api, &e_bit, &f_bit, &g_bit);
			let t1_term3 = bit_array_to_m31(api, &t1_term3_tmp);
			let t1_term4 = bit_array_to_m31(api, &self.kbits[t]); //rewrite to [2]frontend.Variable
			let t1_term5 = bit_array_to_m31(api, &msg_schedule[t]);
			let tmp1 = big_array_add(api, &t1_term1, &t1_term2, 30);
			let tmp2 = big_array_add(api, &t1_term3, &t1_term4, 30);
			let tmp3 = big_array_add(api, &tmp1, &tmp2, 30);
			let tmp4 = big_array_add(api, &tmp3, &t1_term5, 30);
			let t1 = tmp4;
			let t2_tmp1 = cap_sigma0(api, &a_bit);
			let t2_tmp2 = bit_array_to_m31(api, &t2_tmp1);
			let t2_tmp3 = maj(api, &a_bit, &b_bit, &c_bit);
			let t2_tmp4 = bit_array_to_m31(api, &t2_tmp3);
			let t2 = big_array_add(api, &t2_tmp2, &t2_tmp4, 30);
			let new_a_bit_tmp = big_array_add(api, &t1, &t2, 30);
			let new_a_bit = m31_to_bit_array(api, &new_a_bit_tmp)[..32].to_vec();
			let new_e_bit_tmp = big_array_add(api, &d[..2], &t1, 30);
			let new_e_bit = m31_to_bit_array(api, &new_e_bit_tmp)[..32].to_vec();
			h = g.to_vec();
			g = f.to_vec();
			f = e.to_vec();
			d = c.to_vec();
			c = b.to_vec();
			b = a.to_vec();
			a = bit_array_to_m31(api, &new_a_bit).to_vec();
			e = bit_array_to_m31(api, &new_e_bit).to_vec();
			g_bit = f_bit.to_vec();
			f_bit = e_bit.to_vec();
			c_bit = b_bit.to_vec();
			b_bit = a_bit.to_vec();
			a_bit = new_a_bit.to_vec();
			e_bit = new_e_bit.to_vec();
		}
		let hh0_tmp1 = big_array_add(api, &hh[0], &a, 30);
		let hh0_tmp2 = m31_to_bit_array(api, &hh0_tmp1);
		hh[0] = bit_array_to_m31(api, &hh0_tmp2[..32].to_vec()).as_slice().try_into().unwrap();
		let hh1_tmp1 = big_array_add(api, &hh[1], &b, 30);
		let hh1_tmp2 = m31_to_bit_array(api, &hh1_tmp1);
		hh[1] = bit_array_to_m31(api, &hh1_tmp2[..32].to_vec()).as_slice().try_into().unwrap();
		let hh2_tmp1 = big_array_add(api, &hh[2], &c, 30);
		let hh2_tmp2 = m31_to_bit_array(api, &hh2_tmp1);
		hh[2] = bit_array_to_m31(api, &hh2_tmp2[..32].to_vec()).as_slice().try_into().unwrap();
		let hh3_tmp1 = big_array_add(api, &hh[3], &d, 30);
		let hh3_tmp2 = m31_to_bit_array(api, &hh3_tmp1);
		hh[3] = bit_array_to_m31(api, &hh3_tmp2[..32].to_vec()).as_slice().try_into().unwrap();
		let hh4_tmp1 = big_array_add(api, &hh[4], &e, 30);
		let hh4_tmp2 = m31_to_bit_array(api, &hh4_tmp1);
		hh[4] = bit_array_to_m31(api, &hh4_tmp2[..32].to_vec()).as_slice().try_into().unwrap();
		let hh5_tmp1 = big_array_add(api, &hh[5], &f, 30);
		let hh5_tmp2 = m31_to_bit_array(api, &hh5_tmp1);
		hh[5] = bit_array_to_m31(api, &hh5_tmp2[..32].to_vec()).as_slice().try_into().unwrap();
		let hh6_tmp1 = big_array_add(api, &hh[6], &g, 30);
		let hh6_tmp2 = m31_to_bit_array(api, &hh6_tmp1);
		hh[6] = bit_array_to_m31(api, &hh6_tmp2[..32].to_vec()).as_slice().try_into().unwrap();
		let hh7_tmp1 = big_array_add(api, &hh[7], &h, 30);
		let hh7_tmp2 = m31_to_bit_array(api, &hh7_tmp1);
		hh[7] = bit_array_to_m31(api, &hh7_tmp2[..32].to_vec()).as_slice().try_into().unwrap();
		p = &p[CHUNK..];
	}
	hh
	}
}

pub fn sha256_37bytes<C: Config, B:RootAPI<C>>(builder: &mut B, orign_data: &[Variable]) ->Vec<Variable> {
	let mut data = orign_data.to_vec();
	let n = data.len();
	if n != 32+1+4 {
		panic!("len(orignData) !=  32+1+4")
	}
	let mut pre_pad = vec![builder.constant(0); 64-37];
	pre_pad[0] = builder.constant(128);	//0x80
	pre_pad[64-37-2] = builder.constant((37) * 8 / 256);	//length byte
	pre_pad[64-37-1] = builder.constant((32+1+4)*8 - 256);	//length byte
	data.append(&mut pre_pad);	//append padding
	let mut d = MyDigest::new(builder);
	d.reset(builder);
	d.chunk_write(builder, &data);
	d.return_sum(builder).to_vec()
}


declare_circuit!(SHA25637BYTESCircuit {
	input: [Variable;37],
	output: [Variable;32],
});
pub fn check_sha256<C: Config, B: RootAPI<C>>(builder: &mut B, origin_data: &Vec<Variable>) ->Vec<Variable>{
	let output = origin_data[37..].to_vec();
	let result = sha256_37bytes(builder, &origin_data[..37]);
	for i in 0..32 {
		// println!("{}: {:?} {:?}", i, builder.value_of(result[i]), builder.value_of(output[i]));
		builder.assert_is_equal(result[i], output[i]);
	}
	result
}
impl GenericDefine<M31Config> for SHA25637BYTESCircuit<Variable> {
	fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
		for _ in 0..8 {
			let mut data = self.input.to_vec();
			data.append(&mut self.output.to_vec());
			builder.memorized_simple_call(check_sha256, &data);
		}
	}
}



#[test]
fn test_sha256_37bytes(){
	let mut hint_registry = HintRegistry::<M31>::new();
	hint_registry.register("myhint.tobinary", to_binary_hint);
	let compile_result = compile_generic(&SHA25637BYTESCircuit::default(),CompileOptions::default()).unwrap();
	for i in 0..1{
		let data = [i;37];
		let mut hash = Sha256::new();
		hash.update(&data);
		let output = hash.finalize();
		let mut assignment = SHA25637BYTESCircuit::default();
		for i in 0..37 {
			assignment.input[i] = M31::from(data[i] as u32);
		}
		for i in 0..32 {
			assignment.output[i] = M31::from(output[i] as u32);
		}
		let witness = compile_result
			.witness_solver
			.solve_witness_with_hints(&assignment, &mut hint_registry)
			.unwrap();
		let output = compile_result.layered_circuit.run(&witness);
		assert_eq!(output, vec![true]);
	}
}
#[test]
fn debug_sha256_37bytes(){
	let mut hint_registry = HintRegistry::<M31>::new();
	hint_registry.register("myhint.tobinary", to_binary_hint);
	let data = [255;37];
		let mut hash = Sha256::new();
		hash.update(&data);
		let output = hash.finalize();
		let mut assignment = SHA25637BYTESCircuit::default();
		for i in 0..37 {
			assignment.input[i] = M31::from(data[i] as u32);
		}
		for i in 0..32 {
			assignment.output[i] = M31::from(output[i] as u32);
		}
	debug_eval(
        &SHA25637BYTESCircuit::default(),
        &assignment,
        hint_registry,
    );
}