import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'package:esse/utils/adaptive.dart';
import 'package:esse/l10n/localizations.dart';
import 'package:esse/provider.dart';

import 'package:esse/apps/assistant/page.dart';
import 'package:esse/apps/assistant/provider.dart';

class ServiceList extends StatefulWidget {
  const ServiceList({Key key}) : super(key: key);

  @override
  _ServiceListState createState() => _ServiceListState();
}

class _ServiceListState extends State<ServiceList> {
  @override
  Widget build(BuildContext context) {
    final serviceKeys = [1];
    final services = {};

    return Expanded(
      child: ListView.builder(
        itemCount: serviceKeys.length,
        itemBuilder: (BuildContext ctx, int index) => _ListService(),
    ));
  }
}

class _ListService extends StatelessWidget {
  const _ListService({Key key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);
    final isDesktop = isDisplayDesktop(context);

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () {
        final widget = AssistantPage();
        if (isDesktop) {
          Provider.of<AccountProvider>(context, listen: false).updateActivedApp(widget);
        } else {
          Navigator.push(context, MaterialPageRoute(builder: (_) => widget));
        }
      },
      child: Container(
        height: 55.0,
        child: Row(
          children: [
            Container(
              width: 45.0,
              height: 45.0,
              margin: const EdgeInsets.only(left: 20.0, right: 15.0),
              decoration: BoxDecoration(
                image: DecorationImage(
                  image: AssetImage('assets/logo/logo_esse.jpg'),
                  fit: BoxFit.cover,
                ),
                borderRadius: BorderRadius.circular(15.0)
              ),
            ),
            Expanded(
              child: Container(
                height: 55.0,
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Expanded(
                          child: Text('esse',
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(fontSize: 16.0))
                        ),
                        Container(
                          margin: const EdgeInsets.only(left: 15.0, right: 20.0),
                          child: Text('2021-11-12',
                            style: const TextStyle(color: Color(0xFFADB0BB), fontSize: 12.0),
                          ),
                        )
                    ]),
                    SizedBox(height: 5.0),
                    Expanded(
                      child: Text('esse is a echo robot',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(color: Color(0xFFADB0BB), fontSize: 12.0)),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
