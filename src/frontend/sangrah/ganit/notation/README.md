# Notation

`ganit::notation` owns notation-level math surfaces.

Functional anchors today:

- `EquationPart`, `EquationLayout`, and `EquationPartLayout`
- `VectorEquation`, `VectorEquationHandle`, `VectorLatexEquation`, and `VectorTypstEquation`
- `Matrix`, `MatrixCell`, and `MatrixCellLayout`

Add implementation code here when the component is primarily about displaying symbolic math,
equations, or matrices as notation.

Do not put geometric or behavioral math views here. A displayed matrix belongs here; a matrix
transformation belongs in `ganit::linear_algebra`.
