import unittest

from murali_engine import (
    Arrow,
    Axes,
    Axes3D,
    ChatBubble,
    Circle,
    CodeBlock,
    ContextBlock,
    ContextWindow,
    EquationLayout,
    EquationPart,
    Label,
    Latex,
    Letter3D,
    LetterParticles3D,
    Line,
    NumberPlane,
    NumberLine,
    OptimizationPath2D,
    Path,
    ParametricCurve3D,
    ParametricSurface,
    ParticleBelt,
    Prop3D,
    Polygon,
    Rectangle,
    RoundedRectangle,
    Scene,
    SceneView,
    SignalFlow,
    Square,
    Table,
    Matrix,
    Timeline,
    TracedPath,
    Typst,
)

WHITE = (1.0, 1.0, 1.0, 1.0)
GRAY_B = (0.733, 0.733, 0.733, 1.0)
GOLD_C = (0.941, 0.675, 0.373, 1.0)
GREEN = (0.2, 0.8, 0.3, 1.0)


class PythonBindingsTest(unittest.TestCase):
    def test_scene_accepts_basic_tattvas(self) -> None:
        scene = Scene()

        first = scene.add(Label("Hello", height=0.3, color=WHITE), at=(0.0, 1.0, 0.0))
        circle = Circle(radius=0.5, color=GREEN, segments=32).with_stroke(0.03, WHITE)
        self.assertIs(circle.with_stroke(0.03, WHITE), circle)

        second = scene.add(circle, at=(-1.0, 0.0, 0.0))
        scene.add(Square(size=0.6, color=WHITE), at=(1.0, 0.0, 0.0))
        scene.add(Rectangle(width=1.2, height=0.4, color=WHITE), at=(0.0, -1.0, 0.0))

        self.assertEqual(first.id, 1)
        self.assertEqual(second.id, 2)
        self.assertEqual(scene.tattva_count(), 4)

    def test_scene_accepts_chat_input_primitives(self) -> None:
        scene = Scene()

        bubble = ChatBubble(width=5.8, height=0.82, radius=0.18, color=GRAY_B)
        bubble.with_tip("right", 0.42, 0.28)
        bubble.with_tip_inset(0.72)
        bubble.with_stroke(0.018, WHITE)
        button = RoundedRectangle(width=0.34, height=0.34, radius=0.15, color=GOLD_C)
        button.with_stroke(0.01, WHITE)

        bubble_handle = scene.add(bubble)
        button_handle = scene.add(button, at=(2.4, 0.0, 0.05))

        self.assertEqual(bubble_handle.id, 1)
        self.assertEqual(button_handle.id, 2)
        self.assertEqual(scene.tattva_count(), 2)

    def test_scene_accepts_timeline(self) -> None:
        scene = Scene()
        label = scene.add(Label("Hello", height=0.3, color=WHITE), at=(0.0, 0.0, 0.0))

        timeline = Timeline()
        timeline.animate(label).at(0.0).for_duration(0.5).typewrite_text().spawn()

        self.assertEqual(timeline.len(), 1)
        scene.play(timeline)

    def test_scene_accepts_explicit_background(self) -> None:
        scene = Scene(background=WHITE)

        for actual, expected in zip(scene.background(), WHITE, strict=True):
            self.assertAlmostEqual(actual, expected, places=5)
        scene.set_background(GREEN)
        for actual, expected in zip(scene.background(), GREEN, strict=True):
            self.assertAlmostEqual(actual, expected, places=5)
        scene.clear_background()
        self.assertIsNone(scene.background())

    def test_scene_applies_theme_like_objects(self) -> None:
        class Theme:
            background = GREEN

        scene = Scene()

        self.assertIs(scene.apply_theme(Theme()), scene)
        for actual, expected in zip(scene.background(), GREEN, strict=True):
            self.assertAlmostEqual(actual, expected, places=5)

    def test_scene_exposes_video_export_methods(self) -> None:
        scene = Scene()

        self.assertTrue(callable(scene.export))
        self.assertTrue(callable(scene.export_video))

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

        square = scene.add(Square(size=1.0, color=WHITE), at=(2.0, -1.0, 0.0))
        self.assertEqual(scene.position(square), (2.0, -1.0, 0.0))
        min_x, min_y, max_x, max_y = scene.world_bounds(square)
        self.assertAlmostEqual((min_x + max_x) * 0.5, 2.0, places=5)
        self.assertAlmostEqual((min_y + max_y) * 0.5, -1.0, places=5)
        self.assertAlmostEqual(max_x - min_x, 1.0, places=5)
        self.assertAlmostEqual(max_y - min_y, 1.0, places=5)
        self.assertEqual(scene.tattva_count(), 5)

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
        code.with_content_offset(0.0, -0.04)
        scene.add(code, at=(-2.0, 1.4, 0.0))

        scene.add(Latex(r"E = mc^2", height=0.3, color=WHITE), at=(1.8, 1.4, 0.0))
        scene.add(Typst("$x^2 + y^2$", height=0.28, color=WHITE), at=(1.8, 0.8, 0.0))

        axes = Axes(x_range=(-2.0, 2.0), y_range=(-1.5, 1.5), color=GRAY_B)
        axes.with_step(0.5)
        scene.add(axes, at=(-2.0, -1.2, 0.0))

        plane = NumberPlane(x_range=(-2.0, 2.0), y_range=(-1.0, 1.0), step=0.5)
        plane.with_grid_color((0.65, 0.72, 0.80, 0.22))
        plane.with_axis_color((0.84, 0.88, 0.92, 0.38))
        plane.with_grid_thickness(0.008)
        plane.with_axis_thickness(0.018)
        scene.add(plane)

        table = Table([["Layer", "Value"], ["attention", "0.82"]])
        table.with_title("Scores")
        table.with_text_height(0.18)
        self.assertEqual(table.num_rows(), 2)
        self.assertEqual(table.num_cols(), 2)
        scene.add(table, at=(2.2, -1.2, 0.0))

        self.assertEqual(scene.tattva_count(), 7)

    def test_scene_accepts_context_window(self) -> None:
        scene = Scene()

        history = ContextBlock("history", "Conversation history", "user", 4900)
        history.with_preview("Earlier turns and decisions")
        history.truncated_to(2700, "from_start")
        blocks = [
            ContextBlock("instructions", "Core instructions", "system", 620),
            history,
            ContextBlock("tool", "Tool result", "tool", 760),
        ]
        context = ContextWindow(blocks, 8192, title="ASSEMBLED MODEL CONTEXT")
        context_handle = scene.add(context)

        self.assertEqual(context.used_tokens(), 4080)
        self.assertEqual(context.available_tokens(), 4112)
        self.assertEqual(history.omitted_tokens(), 2200)
        self.assertEqual(context_handle.id, 1)
        self.assertEqual(scene.tattva_count(), 1)

    def test_scene_accepts_equations_matrices_and_math_steps(self) -> None:
        scene = Scene()

        source = EquationLayout(
            [
                EquationPart("x", key="x", color=GOLD_C),
                EquationPart("+", key="plus", color=GRAY_B),
                EquationPart("2", key="two", color=WHITE),
            ],
            0.4,
        )
        target = EquationLayout(
            [
                EquationPart("x", key="x", color=GOLD_C),
                EquationPart("=", key="eq", color=GRAY_B),
                EquationPart("3", key="three", color=WHITE),
            ],
            0.4,
        )
        source_handle = scene.add(source)
        target_handle = scene.add(target)
        scene.hide(target_handle)

        number_line = NumberLine((-3.0, 3.0))
        number_line.with_step(1.0)
        number_line.with_color(GRAY_B)
        number_line.with_origin_color(GOLD_C)
        scene.add(number_line, at=(0.0, -0.7, 0.0))

        matrix = Matrix([["1", "0"], ["0", "1"]], 0.4)
        matrix.with_color(WHITE)
        matrix.with_bracket_color(GRAY_B)
        matrix.set_cell_color(0, 0, GOLD_C)
        matrix.set_cell_highlight(0, 0, (1.0, 0.84, 0.04, 0.18))
        matrix_handle = scene.add(matrix, at=(0.0, -1.5, 0.0))

        timeline = Timeline()
        timeline.animate(target_handle).at(0.0).for_duration(0.5).equation_continuity_from(
            source_handle
        ).spawn()
        timeline.animate(matrix_handle).at(0.6).for_duration(0.5).matrix_step_row(
            0, GOLD_C, 0.3
        ).spawn()
        timeline.animate(matrix_handle).at(1.2).for_duration(0.5).matrix_step_column(
            1, WHITE, 0.3
        ).spawn()
        timeline.animate(matrix_handle).at(1.8).for_duration(0.5).matrix_step_cells(
            [(0, 0), (1, 1)], GOLD_C, 0.3
        ).spawn()

        self.assertEqual(timeline.len(), 4)
        self.assertEqual(scene.tattva_count(), 4)
        scene.play(timeline)

    def test_scene_accepts_optimization_path_and_frame_updater(self) -> None:
        scene = Scene()

        arrows = Path.from_points([(-1.0, 0.0), (1.0, 0.0)], color=GRAY_B, thickness=0.03)
        arrows_handle = scene.add(arrows)
        charge = scene.add(Circle(radius=0.16, color=GOLD_C), at=(-0.9, 0.0, 0.0))
        scene.add_updater(lambda scene, time: scene.set_position(charge, (time, 0.0, 0.0)))

        path = OptimizationPath2D([(-1.0, -0.7), (-0.5, -0.4), (0.0, -0.35)])
        path.with_color(GOLD_C)
        path.with_thickness(0.025)
        path.with_point_size(0.07)
        scene.add(path)

        timeline = Timeline()
        timeline.animate(arrows_handle).at(0.0).for_duration(0.5).appear().spawn()

        self.assertEqual(path.steps(), 2)
        self.assertEqual(timeline.len(), 1)
        self.assertEqual(scene.tattva_count(), 3)
        scene.play(timeline)

    def test_scene_accepts_signal_flow_and_camera_timeline(self) -> None:
        scene = Scene()
        scene.set_perspective_camera(fov_y_degrees=45.0)
        scene.set_camera(position=(0.0, 1.2, 8.0), target=(0.0, 0.0, 0.0))

        flow = SignalFlow([(0.0, 0.0, 0.0), (0.8, 0.4, 0.0), (1.2, 0.0, 0.6)])
        flow.with_progress(0.25)
        flow.with_edge_color(GOLD_C)
        flow.with_pulse_color(WHITE)
        flow.with_highlight_nodes(False)
        flow.with_node_radius(0.0)
        flow.with_edge_thickness(0.04)
        flow.with_pulse_radius(0.09)
        handle = scene.add(flow)

        timeline = Timeline()
        timeline.play_signal(handle, 0.0, 1.0, ease="in_out_cubic")
        timeline.animate_camera_frame(
            0.0,
            1.0,
            position=(0.0, 1.6, 7.2),
            target=(0.0, 0.1, 0.0),
            ease="in_out_quad",
        )

        self.assertEqual(timeline.len(), 2)
        self.assertIsNotNone(flow.current_position())
        self.assertEqual(handle.id, 1)
        scene.play(timeline)

    def test_scene_accepts_particle_belt_and_evolve(self) -> None:
        scene = Scene()
        belt = ParticleBelt(2.25)
        belt.with_band_width(0.68)
        belt.with_particle_count(12)
        belt.with_particle_size_range(0.016, 0.055)
        belt.with_palette([GOLD_C, WHITE])
        belt.with_orbit_speed(1.0)
        belt.with_clockwise_ratio(0.35)
        belt.with_band_breathing(0.10, 1.3)
        belt.with_radial_jitter(0.13, 2.7)
        belt.with_seed(7.0)
        handle = scene.add(belt)

        timeline = Timeline()
        timeline.animate(handle).at(0.0).for_duration(0.4).ease("out_cubic").appear().spawn()
        timeline.animate(handle).at(0.2).for_duration(0.8).ease("linear").belt_evolve_with_speed(
            1.05
        ).spawn()
        timeline.zoom_camera(0.0, 0.5, 1.12, ease="in_out_quad")

        self.assertEqual(belt.particle_count(), 12)
        self.assertEqual(belt.radius(), 2.25)
        self.assertEqual(timeline.len(), 3)
        self.assertEqual(scene.tattva_count(), 1)
        scene.play(timeline)

    def test_parametric_surface_from_function_and_update(self) -> None:
        scene = Scene()
        surface = ParametricSurface.from_function(
            lambda u, v: (u, 0.0, v),
            u_range=(0.0, 1.0),
            v_range=(0.0, 1.0),
            samples=(4, 4),
            color=WHITE,
        )
        surface.with_texture("earth")
        surface.with_texture_flip_y(True)
        surface.with_write_progress(0.0)
        handle = scene.add(surface)
        replacement = ParametricSurface.from_function(
            lambda u, v: (u, 0.2, v),
            u_range=(0.0, 1.0),
            v_range=(0.0, 1.0),
            samples=(4, 4),
            color=WHITE,
        )
        replacement.with_write_progress(1.0)
        scene.update_parametric_surface(handle, replacement)
        self.assertEqual(scene.tattva_count(), 1)

    def test_map_projection_surface_and_graticule_are_native(self) -> None:
        scene = Scene()
        surface = ParametricSurface.from_map_projection(
            "equirectangular",
            "sinusoidal",
            mix=0.5,
            samples=(8, 12),
        )
        surface.with_texture("earth")
        handle = scene.add(surface)
        graticule = Path.from_map_graticule("equirectangular", "sinusoidal", mix=0.5)
        scene.add(graticule)
        replacement = ParametricSurface.from_map_projection("mercator", samples=(8, 12))
        scene.update_parametric_surface(handle, replacement)
        self.assertEqual(scene.tattva_count(), 2)

    def test_prop3d_reports_center_and_dimensions(self) -> None:
        from pathlib import Path

        asset = (
            Path(__file__).resolve().parents[2] / "assets" / "props" / "demo-pyramid.glb"
        )
        prop = Prop3D.from_file(str(asset))
        minimum = prop.bounds_min()
        maximum = prop.bounds_max()
        center = prop.center()
        dimensions = prop.dimensions()
        self.assertGreater(prop.mesh_count(), 0)
        self.assertAlmostEqual(center[0], (minimum[0] + maximum[0]) * 0.5, places=5)
        self.assertAlmostEqual(dimensions[0], maximum[0] - minimum[0], places=5)
        scene = Scene()
        scene.add(prop)
        self.assertEqual(scene.tattva_count(), 1)

    def test_prop3d_from_gltf_loads_demo_apple_and_rejects_glb(self) -> None:
        from pathlib import Path

        assets = Path(__file__).resolve().parents[2] / "assets" / "props"
        apple = Prop3D.from_gltf(str(assets / "demo-apple" / "demo-apple.gltf"))
        self.assertGreater(apple.mesh_count(), 0)
        with self.assertRaisesRegex(RuntimeError, "gltf"):
            Prop3D.from_gltf(str(assets / "demo-pyramid.glb"))

    def test_prop3d_from_glb_loads_demo_pyramid_and_rejects_gltf(self) -> None:
        from pathlib import Path

        assets = Path(__file__).resolve().parents[2] / "assets" / "props"
        pyramid = Prop3D.from_glb(str(assets / "demo-pyramid.glb"))
        self.assertGreater(pyramid.mesh_count(), 0)
        with self.assertRaisesRegex(RuntimeError, "glb"):
            Prop3D.from_glb(str(assets / "demo-apple" / "demo-apple.gltf"))

    def test_scene_can_hide_and_show_tattvas(self) -> None:
        scene = Scene()
        label = scene.add(Label("Visibility", height=0.3, color=WHITE))

        scene.hide(label)
        scene.show(label)

        self.assertEqual(scene.tattva_count(), 1)

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

    def test_scene_view_applies_theme_like_objects(self) -> None:
        class Theme:
            background = GREEN

        child = Scene()
        view = SceneView(child)

        self.assertIs(view.apply_theme(Theme()), view)

    def test_scene_accepts_traced_path_path_updates_and_timeline_callbacks(self) -> None:
        scene = Scene()
        scene.set_view_width(11.2)

        points = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0), (1.0, 0.0)]
        path = Path.from_points(points, color=GRAY_B, thickness=0.03)
        path_handle = scene.add(path)
        tip = scene.add(Circle(radius=0.08, color=GOLD_C), at=(1.0, 0.0, 0.0))
        trace = TracedPath(tip, color=GOLD_C, thickness=0.04)
        trace.with_min_distance(0.01)
        trace.with_max_points(400)
        trace_handle = scene.add(trace)

        replacement = Path.from_points(
            [(0.5, 0.0), (0.0, 0.5), (-0.5, 0.0)],
            color=GRAY_B,
            thickness=0.03,
        )
        scene.update_path(path_handle, replacement)

        timeline = Timeline()
        timeline.animate(tip).at(0.0).for_duration(0.3).appear().spawn()
        timeline.call_during(
            0.4,
            1.2,
            lambda scene, t: scene.set_position(tip, (t, 0.0, 0.0)),
            ease="in_out_cubic",
        )
        timeline.call_at(1.6, lambda scene: scene.stop_trace(trace_handle))
        timeline.zoom_camera(0.8, 0.5, 1.4, ease="in_out_cubic")
        scene.play(timeline)

        self.assertEqual(timeline.len(), 4)
        self.assertEqual(scene.tattva_count(), 3)

    def test_typst_outline_points_samples_a_formula(self) -> None:
        points = Typst.outline_points("$pi$", height=1.0, sample_count=32)
        self.assertEqual(len(points), 32)
        self.assertTrue(all(len(point) == 2 for point in points))

    def test_typst_vector_paths_emits_filled_glyphs(self) -> None:
        glyphs = Typst.vector_paths("$x$", height=0.8, color=WHITE)
        self.assertGreater(len(glyphs), 0)
        self.assertTrue(all(glyph.key for glyph in glyphs))
        self.assertTrue(all(len(glyph.center) == 2 for glyph in glyphs))

        scene = Scene()
        handle = scene.add(glyphs[0].path)
        self.assertEqual(scene.tattva_count(), 1)
        self.assertIsNotNone(handle)

    def test_path_morph_from_plays(self) -> None:
        scene = Scene()
        source = scene.add(Path.from_points([(0.0, 0.0), (1.0, 0.0)], color=WHITE, thickness=0.04))
        target = scene.add(Path.from_points([(0.0, 0.5), (1.0, 0.5)], color=GOLD_C, thickness=0.04))
        scene.hide(target)
        timeline = Timeline()
        timeline.animate(target).at(0.0).for_duration(0.3).morph_from(source).spawn()
        scene.play(timeline)
        self.assertEqual(scene.tattva_count(), 2)

    def test_scene_accepts_letter3d_particles_and_scatter(self) -> None:
        scene = Scene()
        letter = Letter3D("K", height=1.2, depth=0.4)
        letter.with_face_colors(WHITE, GRAY_B, GOLD_C)
        letter.with_texture("white_marble")
        solid = scene.add(letter, at=(0.0, 0.0, 0.0))
        scene.set_rotation(solid, 12.0, -8.0, 4.0)

        particles = LetterParticles3D("K", height=1.2, depth=0.4, count=8)
        particles.with_motion(2.0, 1.0, 0.4)
        particles.with_palette([(0.2, 0.8, 0.7, 1.0)])
        cloud = scene.add(particles)
        scene.hide(cloud)

        timeline = Timeline()
        timeline.animate(solid).at(0.0).for_duration(0.4).ease("in_out_cubic").rotate_xyz(
            0.0, 0.0, 0.0
        ).spawn()
        timeline.animate(cloud).at(0.4).for_duration(0.5).letter_particle_scatter_to(1.0).spawn()
        timeline.wait_until(1.2)
        scene.play(timeline)

        self.assertGreater(letter.width(), 0.0)
        self.assertEqual(particles.particle_count(), 8)
        self.assertEqual(scene.tattva_count(), 2)

    def test_scene_can_update_rectangle_and_label_text(self) -> None:
        scene = Scene()
        label = scene.add(Label("Slots", height=0.2, color=WHITE))
        rect = scene.add(Rectangle(width=0.5, height=0.4, color=GRAY_B))
        scene.set_label_text(label, "1 / 6 SLOTS")
        scene.set_label_color(label, GOLD_C)
        scene.update_rectangle(rect, Rectangle(width=0.5, height=0.4, color=GOLD_C))
        self.assertEqual(scene.tattva_count(), 2)


if __name__ == "__main__":
    unittest.main()
