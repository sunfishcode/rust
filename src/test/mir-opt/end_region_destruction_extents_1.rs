// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// compile-flags: -Z identify_regions -Z span_free_formats
// ignore-tidy-linelength

// A scenario with significant destruction code extents (which have
// suffix "dce" in current `-Z identify_regions` rendering).

#![feature(generic_param_attrs)]
#![feature(dropck_eyepatch)]

fn main() {
    // Since the second param to `D1` is may_dangle, it is legal for
    // the region of that parameter to end before the drop code for D1
    // is executed.
    (D1(&S1("ex1"), &S1("dang1"))).0;
}

#[derive(Debug)]
struct S1(&'static str);

#[derive(Debug)]
struct D1<'a, 'b>(&'a S1, &'b S1);

// The `#[may_dangle]` means that references of type `&'b _` may be
// invalid during the execution of this destructor; i.e. in this case
// the destructor code is not allowed to read or write `*self.1`, while
// it can read/write `*self.0`.
unsafe impl<'a, #[may_dangle] 'b> Drop for D1<'a, 'b> {
    fn drop(&mut self) {
        println!("D1({:?}, _)", self.0);
    }
}

// Notes on the MIR output below:
//
// 1. The `EndRegion('16mce)` is allowed to precede the `drop(_3)`
//    solely because of the #[may_dangle] mentioned above.
//
// 2. Regarding the occurrence of `EndRegion('184dce)` *after* `StorageDead(_6)`
//    (where we have borrows `&'184dce _6`): Eventually:
//
//    i. this code should be rejected (by mir-borrowck), or
//
//    ii. the MIR code generation should be changed so that the
//        EndRegion('184dce)` precedes `StorageDead(_6)` in the
//        control-flow.  (Note: arielb1 views drop+storagedead as one
//        unit, and does not see this option as a useful avenue to
//        explore.), or
//
//    iii. the presence of EndRegion should be made irrelevant by a
//        transformation encoding the effects of rvalue-promotion.
//        This may be the simplest and most-likely option; note in
//        particular that `StorageDead(_6)` goes away below in
//        rustc.node4.QualifyAndPromoteConstants.after.mir

// END RUST SOURCE

// START rustc.node4.QualifyAndPromoteConstants.before.mir
// fn main() -> () {
//     let mut _0: ();
//     let mut _1: &'184dce S1;
//     let mut _2: &'184dce S1;
//     let mut _3: D1<'184dce, '16mce>;
//     let mut _4: &'184dce S1;
//     let mut _5: &'184dce S1;
//     let mut _6: S1;
//     let mut _7: &'16mce S1;
//     let mut _8: &'16mce S1;
//     let mut _9: S1;
//
//     bb0: {
//         StorageLive(_2);
//         StorageLive(_3);
//         StorageLive(_4);
//         StorageLive(_5);
//         StorageLive(_6);
//         _6 = S1::{{constructor}}(const "ex1",);
//         _5 = &'184dce _6;
//         _4 = &'184dce (*_5);
//         StorageLive(_7);
//         StorageLive(_8);
//         StorageLive(_9);
//         _9 = S1::{{constructor}}(const "dang1",);
//         _8 = &'16mce _9;
//         _7 = &'16mce (*_8);
//         _3 = D1<'184dce, '16mce>::{{constructor}}(_4, _7);
//         EndRegion('16mce);
//         StorageDead(_7);
//         StorageDead(_4);
//         _2 = (_3.0: &'184dce S1);
//         _1 = _2;
//         StorageDead(_2);
//         drop(_3) -> bb1;
//     }
//
//     bb1: {
//         StorageDead(_3);
//         StorageDead(_8);
//         StorageDead(_9);
//         StorageDead(_5);
//         StorageDead(_6);
//         EndRegion('184dce);
//         _0 = ();
//         return;
//     }
// }
// END rustc.node4.QualifyAndPromoteConstants.before.mir

// START rustc.node4.QualifyAndPromoteConstants.after.mir
// fn main() -> () {
//     let mut _0: ();
//     let mut _1: &'184dce S1;
//     let mut _2: &'184dce S1;
//     let mut _3: D1<'184dce, '16mce>;
//     let mut _4: &'184dce S1;
//     let mut _5: &'184dce S1;
//     let mut _6: S1;
//     let mut _7: &'16mce S1;
//     let mut _8: &'16mce S1;
//     let mut _9: S1;
//
//     bb0: {
//         StorageLive(_2);
//         StorageLive(_3);
//         StorageLive(_4);
//         StorageLive(_5);
//         _5 = promoted1;
//         _4 = &'184dce (*_5);
//         StorageLive(_7);
//         StorageLive(_8);
//         _8 = promoted0;
//         _7 = &'16mce (*_8);
//         _3 = D1<'184dce, '16mce>::{{constructor}}(_4, _7);
//         EndRegion('16mce);
//         StorageDead(_7);
//         StorageDead(_4);
//         _2 = (_3.0: &'184dce S1);
//         _1 = _2;
//         StorageDead(_2);
//         drop(_3) -> bb1;
//     }
//
//     bb1: {
//         StorageDead(_3);
//         StorageDead(_8);
//         StorageDead(_5);
//         EndRegion('184dce);
//         _0 = ();
//         return;
//     }
// }
// END rustc.node4.QualifyAndPromoteConstants.after.mir
