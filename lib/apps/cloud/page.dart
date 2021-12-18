import 'package:flutter/material.dart';

import 'package:esse/l10n/localizations.dart';

class CloudPage extends StatefulWidget {
  const CloudPage({Key? key}) : super(key: key);

  @override
  _CloudPageState createState() => _CloudPageState();
}

class _CloudPageState extends State<CloudPage> {
  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(lang.cloud),
        actions: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20.0),
            child: IconButton(
              icon: Icon(Icons.add),
              onPressed: () {},
            ),
          )
        ]
      ),
      body: Container(
        alignment: Alignment.topCenter,
        child: Text(lang.wip)
      )
    );
  }
}
