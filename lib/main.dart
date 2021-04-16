import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:esse_core/esse_core.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider/device.dart';
import 'package:esse/provider/account.dart';
import 'package:esse/utils/home_dir.dart';
import 'package:esse/theme.dart';
import 'package:esse/global.dart';
import 'package:esse/options.dart';
import 'package:esse/security.dart';
import 'package:esse/rpc.dart';

void coreServer() async {
  final path = await homeDir();
  print("home path: " + path);
  Global.home = path;

  final res = await httpPost(Global.httpRpc, 'echo', []);
  if (res.isOk) {
    print('Had running');
  } else {
    EsseCore.daemon(path);
  }
}

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  coreServer(); // daemon running.
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
      home: SecurityPage(),
    );
  }
}
