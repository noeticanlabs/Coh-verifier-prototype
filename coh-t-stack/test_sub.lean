import Mathlib

abbrev ENNRat := WithTop NNRat

def test_sub (a b : ENNRat) : ENNRat := a - b
