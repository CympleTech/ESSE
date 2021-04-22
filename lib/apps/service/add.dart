import 'package:flutter/material.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/l10n/localizations.dart';

class GroupAddPage extends StatelessWidget {
  const GroupAddPage({Key key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final isDesktop = isDisplayDesktop(context);
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(10.0),
          child: Column(children: <Widget>[
              Row(
                children: [
                  if (!isDesktop)
                  GestureDetector(
                    onTap: () => Navigator.pop(context),
                    child: Container(width: 20.0, child: Icon(Icons.arrow_back, color: color.primary)),
                  ),
                  const SizedBox(width: 15.0),
                  Expanded(child: Text(lang.addGroup, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 20.0))),
                ],
              ),
              Expanded(
                child: Center(
                  child: Text(lang.wip, style: TextStyle(fontSize: 32.0))
                )
              )
            ]
          )
        )
      )
    );
  }
}
