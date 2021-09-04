import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/options.dart';

class PreferenceDetail extends StatefulWidget {
  PreferenceDetail({Key? key}) : super(key: key);

  @override
  _PreferenceDetailState createState() => _PreferenceDetailState();
}

class _PreferenceDetailState extends State<PreferenceDetail> {
  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final options = context.watch<Options>();

    return Column(children: [
      Row(mainAxisAlignment: MainAxisAlignment.spaceEvenly, children: [
        Text(lang.lang,
            style: TextStyle(fontSize: 16.0, fontWeight: FontWeight.bold)),
        Container(
            height: 40.0,
            //width: 200.0,
            padding: EdgeInsets.only(left: 20, right: 20),
            decoration: BoxDecoration(
                color: color.surface,
                borderRadius: BorderRadius.circular(15.0)),
            child: DropdownButtonHideUnderline(
              child: Theme(
                data: Theme.of(context).copyWith(
                  canvasColor: color.surface,
                ),
                child: DropdownButton<Locale>(
                  iconEnabledColor: Color(0xFFADB0BB),
                  value: options.locale,
                  onChanged: (Locale? t) {
                    if (t != options.locale) {
                      options.changeLocale(t!);
                    }
                  },
                  items: AppLocalizations.supportedLocales.map((Locale locale) {
                    return DropdownMenuItem<Locale>(
                      value: locale,
                      child: Text(locale.localizations(),
                          style:
                              TextStyle(color: color.primary, fontSize: 16.0)),
                    );
                  }).toList(),
                ),
              ),
            )),
      ]),
    ]);
  }
}
