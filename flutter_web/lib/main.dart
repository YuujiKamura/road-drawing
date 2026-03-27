import 'package:flutter/material.dart';
import 'app.dart';
import 'wasm_bridge.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  try {
    await WasmBridge.init();
  } catch (e) {
    // WASM load failed — app falls back to Dart preview
    debugPrint('WASM init failed: $e (using Dart fallback)');
  }
  runApp(const RoadDrawingApp());
}

class RoadDrawingApp extends StatelessWidget {
  const RoadDrawingApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Road Drawing',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark(useMaterial3: true).copyWith(
        scaffoldBackgroundColor: const Color(0xFF1A1A1A),
      ),
      home: const MainLayout(),
    );
  }
}
