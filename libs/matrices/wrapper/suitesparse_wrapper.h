#ifndef CSPARSE_WRAPPER_H
#define CSPARSE_WRAPPER_H

#include "cs.h"

css *css_init(const cs *A);

csn *csn_init(const cs *A, const css *S);

int csparse_solve(const css *S, const csn *N, int n, const double *rhs,
                  double *y);

int csparse_matvec(const cs *A, const double *rhs, double *y);

cs *csparse_matmat(const cs *A, const cs *B);

#endif
