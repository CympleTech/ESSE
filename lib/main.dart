import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:window_manager/window_manager.dart';
import 'package:esse_core/esse_core.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/home_dir.dart';
import 'package:esse/theme.dart';
import 'package:esse/global.dart';
import 'package:esse/options.dart';
import 'package:esse/security.dart';
import 'package:esse/rpc.dart';

import 'package:esse/provider.dart';
import 'package:esse/pages/home.dart';
import 'package:esse/apps/device/provider.dart';

void coreServer() async {
  final path = await homeDir();
  print("home path: " + path);
  Global.home = path;

  final res = await httpPost('echo', []);
  if (res.isOk) {
    print('Had running');
  } else {
    EsseCore.daemon(path);
  }
}

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  coreServer(); // daemon running.

  // fix the window size
  await windowManager.ensureInitialized();
  WindowOptions windowOptions = WindowOptions(
    size: Size(1024, 768),
    center: true,
    backgroundColor: Colors.transparent,
    skipTaskbar: false,
    titleBarStyle: TitleBarStyle.hidden,
  );
  windowManager.waitUntilReadyToShow(windowOptions, () async {
      await windowManager.show();
      await windowManager.focus();
  });

  runApp(MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) {
            var option = Options();
            option.load();
            return option;
        }),
        ChangeNotifierProvider(create: (_) => AccountProvider()),
        ChangeNotifierProvider(create: (_) => DeviceProvider()),
      ],
      child: MyApp(),
  ));
}

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final options = context.watch<Options>();

    return MaterialApp(
      title: 'ESSE',
      debugShowCheckedModeBanner: false,
      themeMode: options.themeMode,
      theme: AppTheme.lightThemeData,
      darkTheme: AppTheme.darkThemeData,
      locale: options.locale,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      initialRoute: '/security',
      routes: <String, WidgetBuilder>{
        '/': (BuildContext context) => const HomePage(),
        '/security': (BuildContext context) => const SecurityPage(),
      },
    );
  }
}
