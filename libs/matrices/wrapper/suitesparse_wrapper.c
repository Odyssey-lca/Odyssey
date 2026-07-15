#include "suitesparse_wrapper.h"
#include <stdlib.h>
#include <string.h>

css *css_init(const cs *A) {
  css *res = cs_sqr(1, A, 0);
  return res;
}

csn *csn_init(const cs *A, const css *S) {
  csn *res = cs_lu(A, S, 1e-12);
  return res;
}

int csparse_solve(const css *S, const csn *N, int n, const double *rhs,
                  double *y) {

  // Temporary vector to hold intermediate values
  double *x = cs_malloc(n, sizeof(double));
  if (!x)
    return 0;

  // Apply permutation: x = P * rhs
  cs_ipvec(N->pinv, rhs, x, n); // P * b

  // Solve L * z = x
  cs_lsolve(N->L, x); // L \ x

  // Solve L' * y = z
  cs_usolve(N->U, x); // L' \ x

  // Apply inverse permutation: y = P' * x
  cs_ipvec(S->q, x, y, n); // P' * x

  cs_free(x);
  return 1;
}

int csparse_matvec(const cs *A, const double *rhs, double *y) {
  return cs_gaxpy(A, rhs, y);
}

cs *csparse_matmat(const cs *A, const cs *B) { return cs_multiply(A, B); }
