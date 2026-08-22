# Linear Algebra

Responsibility: vector spaces, bases, coordinates, projections, spans, matrix transformations,
composition, rank/dimension ideas, and geometric linear maps.

Target horizon: build steadily through the end of 2030.

Functional anchors today:

- `VectorArrow2D` and `LabeledVector2D`
- `BasisVectors2D`, `BasisGrid2D`, and `SpanRegion2D`
- `TransformableGrid2D`, `MatrixTransformPanel`, `MatrixVectorFlow`, and `DeterminantAreaView`
- `VectorAdditionView`, `ScalarMultiplicationView`, `LinearCombinationView`, and
  `ColumnCombinationView`
- `ProjectionShadow`, `DotProductMeter`, `AngleArc`, and `OrthogonalityMarker`
- `CoordinateReadout`, `DimensionBadge`, and `QuantityBadge`
- `Matrix`, `MatrixCell`, and `MatrixCellLayout`
- runnable `linear_algebra_*` examples tagged for focused preview and export

This module is available only when the `experimental` feature is enabled.

Add here when a component teaches a reusable linear-algebra concept across scenes.

Do not add scene-shaped explainers here. Prefer smaller pieces such as vectors, grids, readouts,
badges, projection markers, or transformation panels.

Planned component families:

- determinant, area, volume, and orientation views
- row-reduction and elimination views
- eigenspace, diagonalization, and SVD views
- subspace, span, nullspace, column-space, and rank-nullity explainers
