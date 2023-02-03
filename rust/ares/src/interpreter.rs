use self::NockWork::*;
use crate::jets;
use crate::mem::unifying_equality;
use crate::mem::NockStack;
use crate::noun::{Atom, Cell, DirectAtom, IndirectAtom, Noun};
use ares_macros::tas;
use bitvec::prelude::{BitSlice, Lsb0};
use either::Either::*;
use num_traits::cast::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, FromPrimitive, ToPrimitive, Debug)]
#[repr(u64)]
enum NockWork {
    Done,
    NockCellComputeHead,
    NockCellComputeTail,
    NockCellCons,
    Nock0Axis,
    Nock1Constant,
    Nock2ComputeSubject,
    Nock2ComputeFormula,
    Nock2ComputeResult,
    Nock2RestoreSubject,
    Nock3ComputeChild,
    Nock3ComputeType,
    Nock4ComputeChild,
    Nock4Increment,
    Nock5ComputeLeftChild,
    Nock5ComputeRightChild,
    Nock5TestEquals,
    Nock6ComputeTest,
    Nock6ComputeBranch,
    Nock6Done,
    Nock7ComputeSubject,
    Nock7ComputeResult,
    Nock7RestoreSubject,
    Nock8ComputeSubject,
    Nock8ComputeResult,
    Nock8RestoreSubject,
    Nock9ComputeCore,
    Nock9ComputeResult,
    Nock9RestoreSubject,
    Nock10ComputeTree,
    Nock10ComputePatch,
    Nock10Edit,
    Nock11ComputeHint,
    Nock11ComputeResult,
    Nock11Done,
}

fn work_to_noun(work: NockWork) -> Noun {
    unsafe {
        DirectAtom::new_unchecked(work.to_u64().expect("IMPOSSIBLE: work does not fit in u64"))
            .as_atom()
            .as_noun()
    }
}

fn noun_to_work(noun: Noun) -> NockWork {
    if let Left(direct) = noun.as_either_direct_allocated() {
        NockWork::from_u64(direct.data()).expect("Invalid work")
    } else {
        panic!("Work should always be a direct atom.")
    }
}

pub fn interpret(stack: &mut NockStack, mut subject: Noun, formula: Noun) -> Noun {
    let mut res = unsafe { DirectAtom::new_unchecked(0).as_atom().as_noun() };
    stack.push(1);
    unsafe {
        *(stack.local_noun_pointer(0)) = work_to_noun(Done);
    }
    push_formula(stack, formula);
    loop {
        match unsafe { noun_to_work(*(stack.local_noun_pointer(0))) } {
            Done => {
                stack.pop(&mut res);
                break;
            }
            NockCellComputeHead => {
                unsafe {
                    *stack.local_noun_pointer(0) = work_to_noun(NockCellComputeTail);
                    let formula = *stack.local_noun_pointer(1);
                    push_formula(stack, formula);
                };
            }
            NockCellComputeTail => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(NockCellCons);
                    *(stack.local_noun_pointer(1)) = res;
                    let formula = *stack.local_noun_pointer(2);
                    push_formula(stack, formula);
                };
            }
            NockCellCons => {
                unsafe {
                    let head = *stack.local_noun_pointer(1);
                    res = Cell::new(stack, head, res).as_noun();
                };
                stack.pop(&mut res);
            }
            Nock0Axis => {
                if let Ok(atom) = unsafe { (*(stack.local_noun_pointer(1))).as_atom() } {
                    res = slot(subject, atom.as_bitslice());
                    stack.pop(&mut res);
                } else {
                    panic!("Axis must be atom");
                };
            }
            Nock1Constant => {
                unsafe {
                    res = *(stack.local_noun_pointer(1));
                }
                stack.pop(&mut res);
            }
            Nock2ComputeSubject => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock2ComputeFormula);
                    let formula = *stack.local_noun_pointer(1);
                    push_formula(stack, formula);
                };
            }
            Nock2ComputeFormula => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock2ComputeResult);
                    *(stack.local_noun_pointer(1)) = res;
                    let formula = *stack.local_noun_pointer(2);
                    push_formula(stack, formula);
                };
            }
            Nock2ComputeResult => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock2RestoreSubject);
                    *(stack.local_noun_pointer(2)) = subject;
                    subject = *(stack.local_noun_pointer(1));
                    push_formula(stack, res);
                };
            }
            Nock2RestoreSubject => unsafe {
                subject = *(stack.local_noun_pointer(2));
                stack.pop(&mut res);
            },
            Nock3ComputeChild => unsafe {
                *(stack.local_noun_pointer(0)) = work_to_noun(Nock3ComputeType);
                let formula = *stack.local_noun_pointer(1);
                push_formula(stack, formula);
            },
            Nock3ComputeType => {
                res = unsafe {
                    if res.is_cell() {
                        DirectAtom::new_unchecked(0).as_atom().as_noun()
                    } else {
                        DirectAtom::new_unchecked(1).as_atom().as_noun()
                    }
                };
                stack.pop(&mut res);
            }
            Nock4ComputeChild => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock4Increment);
                    let formula = *stack.local_noun_pointer(1);
                    push_formula(stack, formula);
                };
            }
            Nock4Increment => {
                if let Ok(atom) = res.as_atom() {
                    res = inc(stack, atom).as_noun();
                    stack.pop(&mut res);
                } else {
                    panic!("Cannot increment (Nock 4) a cell");
                };
            }
            Nock5ComputeLeftChild => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock5ComputeRightChild);
                    let formula = *stack.local_noun_pointer(1);
                    push_formula(stack, formula);
                };
            }
            Nock5ComputeRightChild => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock5TestEquals);
                    *(stack.local_noun_pointer(1)) = res;
                    let formula = *stack.local_noun_pointer(2);
                    push_formula(stack, formula);
                };
            }
            Nock5TestEquals => {
                unsafe {
                    let saved_value_ptr = stack.local_noun_pointer(1);
                    res = if unifying_equality(stack, &mut res, saved_value_ptr) {
                        DirectAtom::new_unchecked(0).as_atom().as_noun()
                    } else {
                        DirectAtom::new_unchecked(1).as_atom().as_noun()
                    };
                    stack.pop(&mut res);
                };
            }
            Nock6ComputeTest => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock6ComputeBranch);
                    let formula = *stack.local_noun_pointer(1);
                    push_formula(stack, formula);
                };
            }
            Nock6ComputeBranch => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock6Done);
                    if let Left(direct) = res.as_either_direct_allocated() {
                        if direct.data() == 0 {
                            let formula = *stack.local_noun_pointer(2);
                            push_formula(stack, formula);
                        } else if direct.data() == 1 {
                            let formula = *stack.local_noun_pointer(3);
                            push_formula(stack, formula);
                        } else {
                            panic!("Test branch of Nock 6 must return 0 or 1");
                        };
                    } else {
                        panic!("Test branch of Nock 6 must return a direct atom");
                    }
                };
            }
            Nock6Done => {
                stack.pop(&mut res);
            }
            Nock7ComputeSubject => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock7ComputeResult);
                    let formula = *stack.local_noun_pointer(1);
                    push_formula(stack, formula);
                };
            }
            Nock7ComputeResult => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock7RestoreSubject);
                    *(stack.local_noun_pointer(1)) = subject;
                    subject = res;
                    let formula = *stack.local_noun_pointer(2);
                    push_formula(stack, formula);
                };
            }
            Nock7RestoreSubject => {
                unsafe {
                    subject = *(stack.local_noun_pointer(1));
                    stack.pop(&mut res);
                };
            }
            Nock8ComputeSubject => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock8ComputeResult);
                    let formula = *stack.local_noun_pointer(1);
                    push_formula(stack, formula);
                };
            }
            Nock8ComputeResult => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock8RestoreSubject);
                    *(stack.local_noun_pointer(1)) = subject;
                    subject = Cell::new(stack, res, subject).as_noun();
                    let formula = *stack.local_noun_pointer(2);
                    push_formula(stack, formula);
                };
            }
            Nock8RestoreSubject => {
                unsafe {
                    subject = *(stack.local_noun_pointer(1));
                    stack.pop(&mut res);
                };
            }
            Nock9ComputeCore => {
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock9ComputeResult);
                    let formula = *stack.local_noun_pointer(2);
                    push_formula(stack, formula);
                };
            }
            Nock9ComputeResult => {
                unsafe {
                    if let Ok(formula_axis) = (*(stack.local_noun_pointer(1))).as_atom() {
                        *(stack.local_noun_pointer(0)) = work_to_noun(Nock9RestoreSubject);
                        *(stack.local_noun_pointer(2)) = subject;
                        subject = res;
                        push_formula(stack, slot(subject, formula_axis.as_bitslice()));
                    } else {
                        panic!("Axis into core must be atom");
                    }
                };
            }
            Nock9RestoreSubject => unsafe {
                subject = *(stack.local_noun_pointer(2));
                stack.pop(&mut res);
            },
            Nock10ComputeTree => unsafe {
                *(stack.local_noun_pointer(0)) = work_to_noun(Nock10ComputePatch);
                let formula = *stack.local_noun_pointer(3);
                push_formula(stack, formula);
            },
            Nock10ComputePatch => unsafe {
                *(stack.local_noun_pointer(0)) = work_to_noun(Nock10Edit);
                *(stack.local_noun_pointer(3)) = res;
                let formula = *stack.local_noun_pointer(2);
                push_formula(stack, formula);
            },
            Nock10Edit => unsafe {
                if let Ok(edit_axis) = (*stack.local_noun_pointer(1)).as_atom() {
                    let tree = *stack.local_noun_pointer(3);
                    res = edit(stack, edit_axis.as_bitslice(), res, tree);
                    stack.pop(&mut res);
                } else {
                    panic!("Axis into tree must be atom");
                }
            },
            Nock11ComputeHint => unsafe {
                let hint = *stack.local_noun_pointer(1);
                if let Ok(hint_cell) = hint.as_cell() {
                    // match %sham hints, which are scaffolding until we have a real jet dashboard
                    if hint_cell
                        .head()
                        .raw_equals(DirectAtom::new_unchecked(tas!(b"sham")).as_noun())
                    {
                        if let Ok(jet_formula) = hint_cell.tail().as_cell() {
                            let jet_name = jet_formula.tail();
                            if let Ok(jet) = jets::get_jet(jet_name) {
                                res = jet(stack, subject);
                                stack.pop(&mut res);
                                continue;
                            }
                        }
                    }
                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock11ComputeResult);
                    push_formula(stack, hint_cell.tail());
                } else {
                    panic!("IMPOSSIBLE: tried to compute a dynamic hint but hint is an atom");
                }
            },
            Nock11ComputeResult => unsafe {
                *(stack.local_noun_pointer(0)) = work_to_noun(Nock11Done);
                let formula = *stack.local_noun_pointer(2);
                push_formula(stack, formula);
            },
            Nock11Done => {
                stack.pop(&mut res);
            }
        };
    }
    res
}

fn push_formula(stack: &mut NockStack, formula: Noun) {
    if let Ok(formula_cell) = formula.as_cell() {
        // Formula
        match formula_cell.head().as_either_atom_cell() {
            Right(_cell) => {
                stack.push(3);
                unsafe {
                    *(stack.local_noun_pointer(0)) = work_to_noun(NockCellComputeHead);
                    *(stack.local_noun_pointer(1)) = formula_cell.head();
                    *(stack.local_noun_pointer(2)) = formula_cell.tail();
                }
            }
            Left(atom) => {
                if let Ok(direct) = atom.as_direct() {
                    match direct.data() {
                        0 => {
                            stack.push(2);
                            unsafe {
                                *(stack.local_noun_pointer(0)) = work_to_noun(Nock0Axis);
                                *(stack.local_noun_pointer(1)) = formula_cell.tail();
                            };
                        }
                        1 => {
                            stack.push(2);
                            unsafe {
                                *(stack.local_noun_pointer(0)) = work_to_noun(Nock1Constant);
                                *(stack.local_noun_pointer(1)) = formula_cell.tail();
                            };
                        }
                        2 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                stack.push(3);
                                unsafe {
                                    *(stack.local_noun_pointer(0)) =
                                        work_to_noun(Nock2ComputeSubject);
                                    *(stack.local_noun_pointer(1)) = arg_cell.head();
                                    *(stack.local_noun_pointer(2)) = arg_cell.tail();
                                };
                            } else {
                                panic!("Argument for Nock 2 must be cell");
                            };
                        }
                        3 => {
                            stack.push(2);
                            unsafe {
                                *(stack.local_noun_pointer(0)) = work_to_noun(Nock3ComputeChild);
                                *(stack.local_noun_pointer(1)) = formula_cell.tail();
                            };
                        }
                        4 => {
                            stack.push(2);
                            unsafe {
                                *(stack.local_noun_pointer(0)) = work_to_noun(Nock4ComputeChild);
                                *(stack.local_noun_pointer(1)) = formula_cell.tail();
                            };
                        }
                        5 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                stack.push(3);
                                unsafe {
                                    *(stack.local_noun_pointer(0)) =
                                        work_to_noun(Nock5ComputeLeftChild);
                                    *(stack.local_noun_pointer(1)) = arg_cell.head();
                                    *(stack.local_noun_pointer(2)) = arg_cell.tail();
                                };
                            } else {
                                panic!("Argument for Nock 5 must be cell");
                            };
                        }
                        6 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                if let Ok(branch_cell) = arg_cell.tail().as_cell() {
                                    stack.push(4);
                                    unsafe {
                                        *(stack.local_noun_pointer(0)) =
                                            work_to_noun(Nock6ComputeTest);
                                        *(stack.local_noun_pointer(1)) = arg_cell.head();
                                        *(stack.local_noun_pointer(2)) = branch_cell.head();
                                        *(stack.local_noun_pointer(3)) = branch_cell.tail();
                                    }
                                } else {
                                    panic!("Argument tail for Nock 6 must be cell");
                                };
                            } else {
                                panic!("Argument for Nock 6 must be cell");
                            }
                        }
                        7 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                stack.push(3);
                                unsafe {
                                    *(stack.local_noun_pointer(0)) =
                                        work_to_noun(Nock7ComputeSubject);
                                    *(stack.local_noun_pointer(1)) = arg_cell.head();
                                    *(stack.local_noun_pointer(2)) = arg_cell.tail();
                                }
                            } else {
                                panic!("Argument for Nock 7 must be cell");
                            };
                        }
                        8 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                stack.push(3);
                                unsafe {
                                    *(stack.local_noun_pointer(0)) =
                                        work_to_noun(Nock8ComputeSubject);
                                    *(stack.local_noun_pointer(1)) = arg_cell.head();
                                    *(stack.local_noun_pointer(2)) = arg_cell.tail();
                                };
                            } else {
                                panic!("Argument for Nock 8 must be cell");
                            };
                        }
                        9 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                stack.push(3);
                                unsafe {
                                    *(stack.local_noun_pointer(0)) = work_to_noun(Nock9ComputeCore);
                                    *(stack.local_noun_pointer(1)) = arg_cell.head();
                                    *(stack.local_noun_pointer(2)) = arg_cell.tail();
                                };
                            } else {
                                panic!("Argument for Nock 9 must be cell");
                            };
                        }
                        10 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                if let Ok(patch_cell) = arg_cell.head().as_cell() {
                                    stack.push(4);
                                    unsafe {
                                        *(stack.local_noun_pointer(0)) =
                                            work_to_noun(Nock10ComputeTree);
                                        *(stack.local_noun_pointer(1)) = patch_cell.head();
                                        *(stack.local_noun_pointer(2)) = patch_cell.tail();
                                        *(stack.local_noun_pointer(3)) = arg_cell.tail();
                                    };
                                } else {
                                    panic!("Argument head for Nock 10 must be cell");
                                };
                            } else {
                                panic!("Argument for Nock 10 must be cell");
                            };
                        }
                        11 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                stack.push(3);
                                unsafe {
                                    *(stack.local_noun_pointer(0)) =
                                        work_to_noun(if arg_cell.head().is_cell() {
                                            Nock11ComputeHint
                                        } else {
                                            Nock11ComputeResult
                                        });
                                    *(stack.local_noun_pointer(1)) = arg_cell.head();
                                    *(stack.local_noun_pointer(2)) = arg_cell.tail();
                                };
                            } else {
                                panic!("Argument for Nock 11 must be cell");
                            };
                        }
                        _ => {
                            panic!("Invalid opcode");
                        }
                    }
                } else {
                    panic!("Invalid opcode");
                }
            }
        }
    } else {
        panic!("Bad formula: atoms are not formulas");
    }
}

/** Note: axis must fit in a direct atom */
pub fn raw_slot(noun: Noun, axis: u64) -> Noun {
    slot(noun, DirectAtom::new(axis).unwrap().as_bitslice())
}

pub fn slot(mut noun: Noun, axis: &BitSlice<u64, Lsb0>) -> Noun {
    let mut cursor = if let Some(x) = axis.last_one() {
        x
    } else {
        panic!("0 is not allowed as an axis")
    };
    loop {
        if cursor == 0 {
            break;
        };
        cursor -= 1;
        if let Ok(cell) = noun.as_cell() {
            if axis[cursor] {
                noun = cell.tail();
            } else {
                noun = cell.head();
            }
        } else {
            panic!("Axis tried to descend through atom: {:?}", noun);
        };
    }
    noun
}

fn edit(
    stack: &mut NockStack,
    edit_axis: &BitSlice<u64, Lsb0>,
    patch: Noun,
    mut tree: Noun,
) -> Noun {
    let mut res = patch;
    let mut dest: *mut Noun = &mut res;
    let mut cursor = edit_axis
        .last_one()
        .expect("0 is not allowed as an edit axis");
    loop {
        if cursor == 0 {
            unsafe {
                *dest = patch;
            }
            break;
        };
        if let Ok(tree_cell) = tree.as_cell() {
            cursor -= 1;
            if edit_axis[cursor] {
                unsafe {
                    let (cell, cellmem) = Cell::new_raw_mut(stack);
                    *dest = cell.as_noun();
                    (*cellmem).head = tree_cell.head();
                    dest = &mut ((*cellmem).tail);
                }
                tree = tree_cell.tail();
            } else {
                unsafe {
                    let (cell, cellmem) = Cell::new_raw_mut(stack);
                    *dest = cell.as_noun();
                    (*cellmem).tail = tree_cell.tail();
                    dest = &mut ((*cellmem).head);
                }
                tree = tree_cell.tail();
            }
        } else {
            panic!("Invalid axis for edit");
        };
    }
    res
}

fn inc(stack: &mut NockStack, atom: Atom) -> Atom {
    match atom.as_either() {
        Left(direct) => Atom::new(stack, direct.data() + 1),
        Right(indirect) => {
            let indirect_slice = indirect.as_bitslice();
            match indirect_slice.first_zero() {
                None => {
                    // all ones, make an indirect one word bigger
                    let (new_indirect, new_slice) =
                        unsafe { IndirectAtom::new_raw_mut_bitslice(stack, indirect.size() + 1) };
                    new_slice.set(indirect_slice.len(), true);
                    new_indirect.as_atom()
                }
                Some(first_zero) => {
                    let (new_indirect, new_slice) =
                        unsafe { IndirectAtom::new_raw_mut_bitslice(stack, indirect.size()) };
                    new_slice.set(first_zero, true);
                    new_slice[first_zero + 1..]
                        .copy_from_bitslice(&indirect_slice[first_zero + 1..]);
                    new_indirect.as_atom()
                }
            }
        }
    }
}
