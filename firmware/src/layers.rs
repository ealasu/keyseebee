use keyberon_macros::layout;
use keyberon::action::{k, l, m, Action, Action::*, HoldTapConfig};
use keyberon::key_code::KeyCode::*;

pub static LAYERS: keyberon::layout::Layers = layout! {
    {
        [ Tab    Q W E R T n                    n Y U I O P BSpace  ]
        [ n      A S D F G Tab                  n H J K L ; Quote   ]
        [ n      Z X C V B Escape           Enter N M , . / Escape  ]
        [ n n n LGui LShift BSpace LCtrl     RAlt Space (1) - n n n ]
    }
    {
        [ t      ! @ '{' '}' |   n    n PgUp   7 8 9 *    0 ]
        [ t      # $ '(' ')' '`' n    n PgDown 4 5 6 +    n ]
        [ t      % ^ '[' ']' ~   n    n &      1 2 3 '\\' n ]
        [ n      n n t   t   t   t    t t      t t n n    n ]
    }
};


// #[rustfmt::skip]
// pub static LAYERS: keyberon::layout::Layers = &[
//     &[
//         &[k(Tab),     k(Q), k(W),  k(E),    k(R), k(T),    k(Y),     k(U),    k(I),   k(O),    k(P),     k(LBracket)],
//         &[k(RBracket),k(A), k(S),  k(D),    k(F), k(G),    k(H),     k(J),    k(K),   k(L),    k(SColon),k(Quote)   ],
//         &[k(Equal),   k(Z), k(X),  k(C),    k(V), k(B),    k(N),     k(M),    k(Comma),k(Dot), k(Slash), k(Bslash)  ],
//         &[Trans,      Trans,k(LGui),k(LAlt),L1_SP,k(LCtrl),k(RShift),L2_ENTER,k(RAlt),k(BSpace),Trans,   Trans      ],
//     ], &[
//         &[Trans,         k(Pause),Trans,     k(PScreen),Trans,    Trans,Trans,      Trans,  k(Delete),Trans,  Trans,   Trans ],
//         &[Trans,         Trans,   k(NumLock),k(Insert), k(Escape),Trans,k(CapsLock),k(Left),k(Down),  k(Up),  k(Right),Trans ],
//         &[k(NonUsBslash),k(Undo), CUT,       COPY,      PASTE,    Trans,Trans,      k(Home),k(PgDown),k(PgUp),k(End),  Trans ],
//         &[Trans,         Trans,   Trans,     Trans,     Trans,    Trans,Trans,      Trans,  Trans,    Trans,  Trans,   Trans ],
//     ], &[
//         &[s!(Grave),s!(Kb1),s!(Kb2),s!(Kb3),s!(Kb4),s!(Kb5),s!(Kb6),s!(Kb7),s!(Kb8),s!(Kb9),s!(Kb0),s!(Minus)],
//         &[ k(Grave), k(Kb1), k(Kb2), k(Kb3), k(Kb4), k(Kb5), k(Kb6), k(Kb7), k(Kb8), k(Kb9), k(Kb0), k(Minus)],
//         &[a!(Grave),a!(Kb1),a!(Kb2),a!(Kb3),a!(Kb4),a!(Kb5),a!(Kb6),a!(Kb7),a!(Kb8),a!(Kb9),a!(Kb0),a!(Minus)],
//         &[Trans,    Trans,  Trans,  Trans,  CSPACE, Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans    ],
//     ], &[
//         &[k(F1),k(F2),k(F3),k(F4),k(F5),k(F6),k(F7),k(F8),k(F9),k(F10),k(F11),k(F12)],
//         &[Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
//         &[Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
//         &[Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
//     ],
// ];

// const CUT: Action = m(&[LShift, Delete]);
// const COPY: Action = m(&[LCtrl, Insert]);
// const PASTE: Action = m(&[LShift, Insert]);
// const L2_ENTER: Action = HoldTap {
//     timeout: 140,
//     hold: &l(2),
//     tap: &k(Enter),
//     config: keyberon::action::HoldTapConfig::Default,
//     tap_hold_interval: 0
// };
// const L1_SP: Action = HoldTap {
//     timeout: 200,
//     hold: &l(1),
//     tap: &k(Space),
//     config: keyberon::action::HoldTapConfig::Default,
//     tap_hold_interval: 0
// };
// const CSPACE: Action = m(&[LCtrl, Space]);
// macro_rules! s {
//     ($k:ident) => {
//         m(&[LShift, $k])
//     };
// }
// macro_rules! a {
//     ($k:ident) => {
//         m(&[RAlt, $k])
//     };
// }