// pest. The Elegant Parser
// Copyright (C) 2017  Dragoș Tiselice
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(test)]

extern crate test;
extern crate futures;
extern crate pest;

use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use test::Bencher;

use futures::stream::Stream;

use pest::inputs::{Input, Position, StringInput};
use pest::{Parser, ParserState, state};
use pest::streams::ParserStream;
use pest::tokens::Token;

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Rule {
    json,
    object,
    pair,
    array,
    value,
    string,
    escape,
    unicode,
    hex,
    number,
    int,
    exp,
    bool,
    null
}

struct JsonParser;

impl Parser<Rule> for JsonParser {
    fn parse<I: Input + 'static>(rule: Rule, input: Arc<I>) -> ParserStream<Rule, I> {
        fn json<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                          -> Result<Position<I>, Position<I>> {

            state.rule(Rule::json, pos, must_match, |pos, state| {
                value(pos, state, must_match)
            })
        }

        fn object<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                            -> Result<Position<I>, Position<I>> {

            state.rule(Rule::object, pos, must_match, |pos, state| {
                pos.sequence(|p| {
                    p.match_string("{").and_then(|p| {
                        skip(p, state, false)
                    }).and_then(|p| {
                        pair(p, state, false)
                    }).and_then(|p| {
                        skip(p, state, false)
                    }).and_then(|p| {
                        p.repeat(|p| {
                            p.sequence(|p| {
                                p.match_string(",").and_then(|p| {
                                    skip(p, state, false)
                                }).and_then(|p| {
                                    pair(p, state, false)
                                }).and_then(|p| {
                                    skip(p, state, false)
                                })
                            })
                        })
                    }).and_then(|p| {
                        p.match_string("}")
                    })
                }).or_else(|p| {
                    p.sequence(|p| {
                        p.match_string("{").and_then(|p| {
                            skip(p, state, must_match)
                        }).and_then(|p| {
                            p.match_string("}")
                        })
                    })
                })
            })
        }

        fn pair<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                          -> Result<Position<I>, Position<I>> {

            state.rule(Rule::pair, pos, must_match, |pos, state| {
                string(pos, state, must_match).and_then(|p| {
                    skip(p, state, must_match)
                }).and_then(|p| {
                    p.match_string(":")
                }).and_then(|p| {
                    skip(p, state, must_match)
                }).and_then(|p| {
                    value(p, state, must_match)
                })
            })
        }

        fn array<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                           -> Result<Position<I>, Position<I>> {

            state.rule(Rule::array, pos, must_match, |pos, state| {
                pos.sequence(|p| {
                    p.match_string("[").and_then(|p| {
                        skip(p, state, false)
                    }).and_then(|p| {
                        value(p, state, false)
                    }).and_then(|p| {
                        skip(p, state, false)
                    }).and_then(|p| {
                        p.repeat(|p| {
                            p.sequence(|p| {
                                p.match_string(",").and_then(|p| {
                                    skip(p, state, false)
                                }).and_then(|p| {
                                    value(p, state, false)
                                }).and_then(|p| {
                                    skip(p, state, false)
                                })
                            })
                        })
                    }).and_then(|p| {
                        p.match_string("]")
                    })
                }).or_else(|p| {
                    p.sequence(|p| {
                        p.match_string("[").and_then(|p| {
                            skip(p, state, must_match)
                        }).and_then(|p| {
                            p.match_string("]")
                        })
                    })
                })
            })
        }

        fn value<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                           -> Result<Position<I>, Position<I>> {

            state.rule(Rule::value, pos, must_match, |pos, state| {
                string(pos, state, false).or_else(|p| {
                    number(p, state, false)
                }).or_else(|p| {
                    object(p, state, false)
                }).or_else(|p| {
                    array(p, state, false)
                }).or_else(|p| {
                    bool(p, state, false)
                }).or_else(|p| {
                    null(p, state, must_match)
                })
            })
        }

        fn string<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                            -> Result<Position<I>, Position<I>> {

            state.rule(Rule::string, pos, must_match, |pos, state| {
                pos.sequence(|p| {
                    p.match_string("\"").and_then(|p| {
                        p.repeat(|p| {
                            escape(p, state, false).or_else(|p| {
                                p.sequence(|p| {
                                    p.negate(|p| {
                                        state.lookahead(false, move |_| {
                                            p.lookahead(|p| {
                                                p.match_string("\"").or_else(|p| {
                                                    p.match_string("\\")
                                                })
                                            })
                                        })
                                    }).and_then(|p| {
                                        p.skip(1)
                                    })
                                })
                            })
                        })
                    }).and_then(|p| {
                        p.match_string("\"")
                    })
                })
            })
        }

        fn escape<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                            -> Result<Position<I>, Position<I>> {

            pos.sequence(|p| {
                p.match_string("\\").and_then(|p| {
                    p.match_string("\"").or_else(|p| {
                        p.match_string("\\")
                    }).or_else(|p| {
                        p.match_string("/")
                    }).or_else(|p| {
                        p.match_string("b")
                    }).or_else(|p| {
                        p.match_string("f")
                    }).or_else(|p| {
                        p.match_string("n")
                    }).or_else(|p| {
                        p.match_string("r")
                    }).or_else(|p| {
                        p.match_string("t")
                    }).or_else(|p| {
                        unicode(p, state, must_match)
                    })
                })
            })
        }

        fn unicode<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                             -> Result<Position<I>, Position<I>> {

            pos.sequence(|p| {
                p.match_string("u").and_then(|p| {
                    hex(p, state, must_match)
                }).and_then(|p| {
                    hex(p, state, must_match)
                }).and_then(|p| {
                    hex(p, state, must_match)
                })
            })
        }

        fn hex<I: Input>(pos: Position<I>, _: &mut ParserState<Rule, I>, _: bool)
                         -> Result<Position<I>, Position<I>> {

            pos.match_range('0'..'9').or_else(|p| {
                p.match_range('a'..'f')
            }).or_else(|p| {
                p.match_range('A'..'F')
            })
        }

        fn number<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                            -> Result<Position<I>, Position<I>> {

            state.rule(Rule::number, pos, must_match, |pos, state| {
                pos.sequence(|p| {
                    p.optional(|p| {
                        p.match_string("-")
                    }).and_then(|p| {
                        int(p, state, must_match)
                    }).and_then(|p| {
                        p.optional(|p| {
                            p.sequence(|p| {
                                p.match_string(".").and_then(|p| {
                                    p.match_range('0'..'9')
                                }).and_then(|p| {
                                    p.repeat(|p| {
                                        p.match_range('0'..'9')
                                    })
                                }).and_then(|p| {
                                    p.optional(|p| {
                                        exp(p, state, false)
                                    })
                                }).or_else(|p| {
                                    exp(p, state, false)
                                })
                            })
                        })
                    })
                })
            })
        }

        fn int<I: Input>(pos: Position<I>, _: &mut ParserState<Rule, I>, _: bool)
                         -> Result<Position<I>, Position<I>> {

            pos.match_string("0").or_else(|p| {
                p.sequence(|p| {
                    p.match_range('1'..'9').and_then(|p| {
                        p.repeat(|p| {
                            p.match_range('0'..'9')
                        })
                    })
                })
            })
        }

        fn exp<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                         -> Result<Position<I>, Position<I>> {

            pos.sequence(|p| {
                p.match_string("E").or_else(|p| {
                    p.match_string("e")
                }).and_then(|p| {
                    p.optional(|p| {
                        p.match_string("+").or_else(|p| {
                            p.match_string("-")
                        })
                    })
                }).and_then(|p| {
                    int(p, state,  must_match)
                })
            })
        }

        fn bool<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                          -> Result<Position<I>, Position<I>> {

            state.rule(Rule::bool, pos, must_match, |pos, state| {
                pos.match_string("true").or_else(|p| {
                    p.match_string("false")
                })
            })
        }

        fn null<I: Input>(pos: Position<I>, state: &mut ParserState<Rule, I>, must_match: bool)
                          -> Result<Position<I>, Position<I>> {

            state.rule(Rule::null, pos, must_match, |pos, state| {
                pos.match_string("null")
            })
        }

        fn skip<I: Input>(pos: Position<I>, _: &mut ParserState<Rule, I>, _: bool)
                          -> Result<Position<I>, Position<I>> {

            pos.repeat(|p| {
                p.match_string(" ").or_else(|p| {
                    p.match_string("\t")
                }).or_else(|p| {
                    p.match_string("\r")
                }).or_else(|p| {
                    p.match_string("\n")
                })
            })
        }

        state(input, move |mut state| {
            if match rule {
                Rule::json    =>    json(state.start(), &mut state, true),
                Rule::object  =>  object(state.start(), &mut state, true),
                Rule::pair    =>    pair(state.start(), &mut state, true),
                Rule::array   =>   array(state.start(), &mut state, true),
                Rule::value   =>   value(state.start(), &mut state, true),
                Rule::string  =>  string(state.start(), &mut state, true),
                Rule::escape  =>  escape(state.start(), &mut state, true),
                Rule::unicode => unicode(state.start(), &mut state, true),
                Rule::hex     =>     hex(state.start(), &mut state, true),
                Rule::number  =>  number(state.start(), &mut state, true),
                Rule::int     =>     int(state.start(), &mut state, true),
                Rule::exp     =>     exp(state.start(), &mut state, true),
                Rule::bool    =>    bool(state.start(), &mut state, true),
                Rule::null    =>    null(state.start(), &mut state, true)
            }.is_err() {
                state.fail_with_attempts();
            }
        })
    }
}

#[bench]
fn data(b: &mut Bencher) {
    let mut file = File::open("benches/data.json").unwrap();
    let mut data = String::new();

    file.read_to_string(&mut data).unwrap();

    let input = Arc::new(StringInput::new(data.to_owned()));

    b.iter(|| {
        JsonParser::parse(Rule::json, input.clone()).wait().count()
    });
}
