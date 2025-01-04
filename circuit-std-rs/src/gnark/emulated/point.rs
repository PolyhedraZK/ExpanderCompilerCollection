use crate::gnark::limbs::*;
use crate::gnark::utils::*;
use crate::gnark::emparam::FieldParams;
use crate::gnark::element::*;
/*
type AffinePoint[Base emulated.FieldParams] struct {
	X, Y emulated.Element[Base]
}
*/
pub struct AffinePoint<Base: FieldParams> {
    pub X: Element<Base>,
    pub Y: Element<Base>,
}

/*
type Curve[Base, Scalars emulated.FieldParams] struct {
	// params is the parameters of the curve
	params CurveParams
	// api is the native api, we construct it ourselves to be sure
	api frontend.API
	// baseApi is the api for point operations
	baseApi *emulated.Field[Base]
	// scalarApi is the api for scalar operations
	scalarApi *emulated.Field[Scalars]

	// g is the generator (base point) of the curve.
	g AffinePoint[Base]

	// gm are the pre-computed doubles the generator (base point) of the curve.
	gm []AffinePoint[Base]

	a            emulated.Element[Base]
	b            emulated.Element[Base]
	addA         bool
	eigenvalue   *emulated.Element[Scalars]
	thirdRootOne *emulated.Element[Base]
}
*/
// pub struct Curve<Base: FieldParams, Scalars: FieldParams> {
//     pub params: CurveParams,
//     pub api: Box<dyn API>,
//     pub baseApi: Box<dyn Field<Base>>,
//     pub scalarApi: Box<dyn Field<Scalars>>,
//     pub g: AffinePoint<Base>,
//     pub gm: Vec<AffinePoint<Base>>,
//     pub a: Element<Base>,
//     pub b: Element<Base>,
//     pub addA: bool,
//     pub eigenvalue: Option<Element<Scalars>>,
//     pub thirdRootOne: Option<Element<Base>>,
// }

/*
func New[Base, Scalars emulated.FieldParams](api frontend.API, params CurveParams) (*Curve[Base, Scalars], error) {
	ba, err := emulated.NewField[Base](api)
	if err != nil {
		return nil, fmt.Errorf("new base api: %w", err)
	}
	sa, err := emulated.NewField[Scalars](api)
	if err != nil {
		return nil, fmt.Errorf("new scalar api: %w", err)
	}
	emuGm := make([]AffinePoint[Base], len(params.Gm))
	for i, v := range params.Gm {
		emuGm[i] = AffinePoint[Base]{emulated.ValueOf[Base](v[0]), emulated.ValueOf[Base](v[1])}
	}
	Gx := emulated.ValueOf[Base](params.Gx)
	Gy := emulated.ValueOf[Base](params.Gy)
	var eigenvalue *emulated.Element[Scalars]
	var thirdRootOne *emulated.Element[Base]
	if params.Eigenvalue != nil && params.ThirdRootOne != nil {
		eigenvalue = sa.NewElement(params.Eigenvalue)
		thirdRootOne = ba.NewElement(params.ThirdRootOne)
	}
	return &Curve[Base, Scalars]{
		params:    params,
		api:       api,
		baseApi:   ba,
		scalarApi: sa,
		g: AffinePoint[Base]{
			X: Gx,
			Y: Gy,
		},
		gm:           emuGm,
		a:            emulated.ValueOf[Base](params.A),
		b:            emulated.ValueOf[Base](params.B),
		addA:         params.A.Cmp(big.NewInt(0)) != 0,
		eigenvalue:   eigenvalue,
		thirdRootOne: thirdRootOne,
	}, nil
}
// */

// pub fn New<Base: FieldParams, Scalars: FieldParams>(api: &mut dyn API, params: CurveParams) -> Result<Curve<Base, Scalars>, String> {
//     let ba = NewField::<Base>(api)?;
//     let sa = NewField::<Scalars>(api)?;
//     let mut emuGm = vec![];
//     for v in params.Gm.iter() {
//         emuGm.push(AffinePoint{X: ValueOf::<Base>(v[0]), Y: ValueOf::<Base>(v[1])});
//     }
//     let Gx = ValueOf::<Base>(params.Gx);
//     let Gy = ValueOf::<Base>(params.Gy);
//     let mut eigenvalue = None;
//     let mut thirdRootOne = None;
//     if let (Some(eigenvalue_val), Some(thirdRootOne_val)) = (params.Eigenvalue, params.ThirdRootOne) {
//         eigenvalue = Some(sa.NewElement(eigenvalue_val));
//         thirdRootOne = Some(ba.NewElement(thirdRootOne_val));
//     }
//     Ok(Curve {
//         params,
//         api,
//         baseApi: ba,
//         scalarApi: sa,
//         g: AffinePoint {
//             X: Gx,
//             Y: Gy,
//         },
//         gm: emuGm,
//         a: ValueOf::<Base>(params.A),
//         b: ValueOf::<Base>(params.B),
//         addA: params.A != BigInt::from(0),
//         eigenvalue,
//         thirdRootOne,
//     })
// }