import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/global.dart';

class AboutDetail extends StatefulWidget {
  AboutDetail({Key? key}) : super(key: key);

  @override
  _AboutDetailState createState() => _AboutDetailState();
}

class _AboutDetailState extends State<AboutDetail> {
  final websiteUrl = 'https://cympletech.com';
  final githubUrl = 'https://github.com/cympletech/esse';
  final twitterUrl = 'https://twitter.com/cympletech';
  final emailUrl = 'mailto:contact@cympletech.com?subject=Hi&body=Hello Esse';

  @override
  Widget build(BuildContext context) {
    final lang = AppLocalizations.of(context);
    return Column(
      children: [
        Text('ESSE ' + Global.version, style: Theme.of(context).textTheme.headline6),
        const SizedBox(height: 10.0),
        Text(lang.title, style: Theme.of(context).textTheme.headline6),
        const SizedBox(height: 10.0),
        Text(lang.about2, style: Theme.of(context).textTheme.headline6),
        const SizedBox(height: 15.0),
        // Container(
        //   width: 400.0,
        //   padding: const EdgeInsets.symmetric(vertical: 5.0),
        //   child: ListTile(
        //     leading: Icon(Icons.favorite),
        //     title: Text(lang.donate, textAlign: TextAlign.center),
        //   ),
        // ),
        Container(
          width: 400.0,
          padding: const EdgeInsets.symmetric(vertical: 5.0),
          child: Tooltip(
            message: websiteUrl,
            child: ListTile(
              leading: Icon(Icons.language),
              title: Text(lang.website, textAlign: TextAlign.center),
              onTap: () => launch(websiteUrl)
        ))),
        Container(
          width: 400.0,
          padding: const EdgeInsets.symmetric(vertical: 5.0),
          child: Tooltip(
            message: 'contact@cympletech.com',
            child: ListTile(
              leading: Icon(Icons.email),
              title: Text(lang.email, textAlign: TextAlign.center),
              onTap: () => launch(emailUrl)
        ))),
        Container(
          width: 400.0,
          padding: const EdgeInsets.symmetric(vertical: 5.0),
          child: Tooltip(
            message: githubUrl,
            child: ListTile(
              leading: Icon(Icons.source),
              title: Text('Github', textAlign: TextAlign.center),
              onTap: () => launch(githubUrl)
        ))),
        Container(
          width: 400.0,
          padding: const EdgeInsets.symmetric(vertical: 5.0),
          child: Tooltip(
            message: twitterUrl,
            child: ListTile(
              leading: Icon(Icons.thumb_up),
              title: Text('Twitter', textAlign: TextAlign.center),
              onTap: () => launch(twitterUrl)
        ))),
      ]
    );
  }
}
