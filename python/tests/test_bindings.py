import unittest

from murali_engine import (
    GOLD_C,
    GRAY_B,
    GREEN,
    WHITE,
    Arrow,
    Axes,
    Axes3D,
    Circle,
    CodeBlock,
    Label,
    Latex,
    Line,
    NumberPlane,
    Path,
    ParametricCurve3D,
    ParametricSurface,
    Polygon,
    Rectangle,
    Scene,
    SceneView,
    Square,
    Table,
    Timeline,
    Typst,
)


class PythonBindingsTest(unittest.TestCase):
    def test_scene_accepts_basic_tattvas(self) -> None:
        scene = Scene()

        first = scene.add(Label("Hello", height=0.3, color=WHITE), at=(0.0, 1.0, 0.0))
        circle = Circle(radius=0.5, color=GREEN, segments=32)
        circle.with_stroke(0.03, WHITE)

        second = scene.add(circle, at=(-1.0, 0.0, 0.0))
        scene.add(Square(size=0.6, color=WHITE), at=(1.0, 0.0, 0.0))
        scene.add(Rectangle(width=1.2, height=0.4, color=WHITE), at=(0.0, -1.0, 0.0))

        self.assertEqual(first.id, 1)
        self.assertEqual(second.id, 2)
        self.assertEqual(scene.tattva_count(), 4)

    def test_scene_accepts_timeline(self) -> None:
        scene = Scene()
        label = scene.add(Label("Hello", height=0.3, color=WHITE), at=(0.0, 0.0, 0.0))

        timeline = Timeline()
        timeline.animate(label).at(0.0).for_duration(0.5).typewrite_text().spawn()

        self.assertEqual(timeline.len(), 1)
        scene.play(timeline)

    def test_scene_accepts_geometry_primitives_and_layout_helpers(self) -> None:
        scene = Scene(frame="portrait")

        title = scene.add(Label("Geometry", height=0.3, color=WHITE))
        scene.to_edge(title, "up", 0.6)
        self.assertEqual(scene.frame(), "portrait")
        self.assertEqual(scene.frame_size(), (9.0, 16.0))

        polygon = Polygon.regular(6, radius=0.7, color=GOLD_C)
        polygon.with_stroke(0.03, WHITE)
        hexagon = scene.add(polygon, at=(-1.5, 0.0, 0.0))

        line = Line(start=(-0.5, -1.0, 0.0), end=(0.5, -1.0, 0.0), color=GRAY_B)
        line.with_dash(0.12, 0.08)
        baseline = scene.add(line)

        arrow = scene.add(Arrow(start=(0.8, -1.0), end=(1.8, -1.0), color=WHITE))
        scene.next_to(arrow, baseline, "right", 0.2)
        scene.align_to(hexagon, title, "center")

        self.assertEqual(scene.tattva_count(), 4)

    def test_timeline_accepts_additional_animation_aliases(self) -> None:
        scene = Scene()
        square = scene.add(Square(size=0.7, color=WHITE))
        label = scene.add(Label("Pulse", height=0.24, color=WHITE))

        timeline = Timeline()
        timeline.animate(square).at(0.0).for_duration(0.4).draw().spawn()
        timeline.animate(square).at(0.5).for_duration(0.4).rotate_to(45.0).spawn()
        timeline.animate(label).at(0.8).for_duration(0.3).indicate().spawn()
        timeline.animate(label).at(1.2).for_duration(0.3).hide_text().spawn()
        table = scene.add(Table([["A", "B"]]))
        timeline.animate(table).at(1.5).for_duration(0.3).write_table().spawn()

        self.assertEqual(timeline.len(), 5)
        scene.play(timeline)

    def test_scene_accepts_path_text_math_axes_and_table(self) -> None:
        scene = Scene()

        path = Path(color=GOLD_C, thickness=0.04)
        path.move_to((-1.0, 0.0))
        path.cubic_to((-0.5, 0.9), (0.5, -0.9), (1.0, 0.0))
        path.with_dash(0.12, 0.08)
        scene.add(path)

        code = CodeBlock("print('murali')", language="python", font_size=0.18)
        code.with_title("demo.py")
        code.with_line_numbers(True)
        scene.add(code, at=(-2.0, 1.4, 0.0))

        scene.add(Latex(r"E = mc^2", height=0.3, color=WHITE), at=(1.8, 1.4, 0.0))
        scene.add(Typst("$x^2 + y^2$", height=0.28, color=WHITE), at=(1.8, 0.8, 0.0))

        axes = Axes(x_range=(-2.0, 2.0), y_range=(-1.5, 1.5), color=GRAY_B)
        axes.with_step(0.5)
        scene.add(axes, at=(-2.0, -1.2, 0.0))

        scene.add(NumberPlane(x_range=(-2.0, 2.0), y_range=(-1.0, 1.0), step=0.5))

        table = Table([["Layer", "Value"], ["attention", "0.82"]])
        table.with_title("Scores")
        table.with_text_height(0.18)
        self.assertEqual(table.num_rows(), 2)
        self.assertEqual(table.num_cols(), 2)
        scene.add(table, at=(2.2, -1.2, 0.0))

        self.assertEqual(scene.tattva_count(), 7)

    def test_scene_accepts_3d_camera_and_scene_view_surfaces(self) -> None:
        scene = Scene()
        scene.set_perspective_camera(fov_y_degrees=45.0)
        scene.set_camera(position=(0.0, 1.2, 8.0), target=(0.0, 0.0, 0.0))

        axes = scene.add(Axes3D(), at=(0.0, 0.0, 0.0))
        curve = scene.add(ParametricCurve3D.named("helix"), at=(0.0, 0.0, 0.0))
        surface = ParametricSurface.named("saddle", render_mode="wireframe")
        surface.with_write_progress(0.5)
        surface_handle = scene.add(surface, at=(0.0, -0.4, 0.0))
        scene.set_depth_mode(axes, "world")
        scene.set_depth_mode(curve, "world")
        scene.set_depth_mode(surface_handle, "world")

        overlay = scene.add(Label("Overlay", height=0.2, color=WHITE))
        scene.set_depth_mode(overlay, "overlay")

        child = Scene()
        child.add(Label("Child", height=0.2, color=WHITE))
        view = SceneView(child)
        view.size(2.0, 1.0)
        view.playback("once")
        scene.add_scene_view(view, at=(2.5, -1.5, 0.0))

        self.assertEqual(scene.tattva_count(), 5)


if __name__ == "__main__":
    unittest.main()
