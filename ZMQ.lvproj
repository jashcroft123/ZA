<?xml version='1.0' encoding='UTF-8'?>
<Project Type="Project" LVVersion="25008000">
	<Property Name="NI.LV.All.SaveVersion" Type="Str">25.0</Property>
	<Property Name="NI.LV.All.SourceOnly" Type="Bool">true</Property>
	<Item Name="My Computer" Type="My Computer">
		<Property Name="NI.SortType" Type="Int">3</Property>
		<Property Name="server.app.propertiesEnabled" Type="Bool">true</Property>
		<Property Name="server.control.propertiesEnabled" Type="Bool">true</Property>
		<Property Name="server.tcp.enabled" Type="Bool">false</Property>
		<Property Name="server.tcp.port" Type="Int">0</Property>
		<Property Name="server.tcp.serviceName" Type="Str">My Computer/VI Server</Property>
		<Property Name="server.tcp.serviceName.default" Type="Str">My Computer/VI Server</Property>
		<Property Name="server.vi.callsEnabled" Type="Bool">true</Property>
		<Property Name="server.vi.propertiesEnabled" Type="Bool">true</Property>
		<Property Name="specify.custom.address" Type="Bool">false</Property>
		<Item Name="Examples" Type="Folder">
			<Item Name="Pair.vi" Type="VI" URL="../src/Examples/Pair.vi"/>
			<Item Name="ReqRep.vi" Type="VI" URL="../src/Examples/ReqRep.vi"/>
			<Item Name="PubSub.vi" Type="VI" URL="../src/Examples/PubSub.vi"/>
			<Item Name="PubSubSync.vi" Type="VI" URL="../src/Examples/PubSubSync.vi"/>
			<Item Name="PushPull.vi" Type="VI" URL="../src/Examples/PushPull.vi"/>
		</Item>
		<Item Name="Example.SubChat.vi" Type="VI" URL="../Example.SubChat.vi"/>
		<Item Name="Example.vi" Type="VI" URL="../Example.vi"/>
		<Item Name="Linux.Test.vi" Type="VI" URL="../Linux.Test.vi"/>
		<Item Name="ExampleClient.vi" Type="VI" URL="../src/ExampleClient.vi"/>
		<Item Name="ZeroMQ.lvlib" Type="Library" URL="../src/zeromq/ZeroMQ.lvlib"/>
		<Item Name="LibZMQ.lvlib" Type="Library" URL="../libzmq-v143-mt-4_3_6/LibZMQ.lvlib"/>
		<Item Name="ConnectToSplash.vi" Type="VI" URL="../src/Examples/ConnectToSplash.vi"/>
		<Item Name="StartupSplash.lvclass" Type="LVClass" URL="../src/StartupSplash/StartupSplash.lvclass"/>
		<Item Name="Dependencies" Type="Dependencies"/>
		<Item Name="Build Specifications" Type="Build">
			<Item Name="Zero Launched App" Type="EXE">
				<Property Name="App_copyErrors" Type="Bool">true</Property>
				<Property Name="App_INI_aliasGUID" Type="Str">{78DDE413-88D4-4927-A36E-0EF60C09CE89}</Property>
				<Property Name="App_INI_GUID" Type="Str">{B2C1C88C-DA6A-40D9-8EBE-FDA8BDF9F02D}</Property>
				<Property Name="App_serverConfig.httpPort" Type="Int">8002</Property>
				<Property Name="App_serverType" Type="Int">0</Property>
				<Property Name="Bld_autoIncrement" Type="Bool">true</Property>
				<Property Name="Bld_buildCacheID" Type="Str">{318FBA16-9475-4961-992C-FED7DB7DF4D5}</Property>
				<Property Name="Bld_buildSpecName" Type="Str">Zero Launched App</Property>
				<Property Name="Bld_excludeInlineSubVIs" Type="Bool">true</Property>
				<Property Name="Bld_excludeLibraryItems" Type="Bool">true</Property>
				<Property Name="Bld_excludePolymorphicVIs" Type="Bool">true</Property>
				<Property Name="Bld_localDestDir" Type="Path">../builds/NI_AB_PROJECTNAME/Zero Launched App</Property>
				<Property Name="Bld_localDestDirType" Type="Str">relativeToCommon</Property>
				<Property Name="Bld_modifyLibraryFile" Type="Bool">true</Property>
				<Property Name="Bld_previewCacheID" Type="Str">{97377283-6FD7-4674-9C79-7CAABC26AF99}</Property>
				<Property Name="Bld_version.build" Type="Int">6</Property>
				<Property Name="Bld_version.major" Type="Int">1</Property>
				<Property Name="Destination[0].destName" Type="Str">Zero Launched App.exe</Property>
				<Property Name="Destination[0].path" Type="Path">../builds/NI_AB_PROJECTNAME/Zero Launched App/Zero Launched App.exe</Property>
				<Property Name="Destination[0].preserveHierarchy" Type="Bool">true</Property>
				<Property Name="Destination[0].type" Type="Str">App</Property>
				<Property Name="Destination[1].destName" Type="Str">Support Directory</Property>
				<Property Name="Destination[1].path" Type="Path">../builds/NI_AB_PROJECTNAME/Zero Launched App/data</Property>
				<Property Name="DestinationCount" Type="Int">2</Property>
				<Property Name="Source[0].itemID" Type="Str">{36DF9CCE-EF83-4102-BE11-620E1448DC45}</Property>
				<Property Name="Source[0].type" Type="Str">Container</Property>
				<Property Name="Source[1].destinationIndex" Type="Int">0</Property>
				<Property Name="Source[1].itemID" Type="Ref">/My Computer/ConnectToSplash.vi</Property>
				<Property Name="Source[1].sourceInclusion" Type="Str">TopLevel</Property>
				<Property Name="Source[1].type" Type="Str">VI</Property>
				<Property Name="Source[2].destinationIndex" Type="Int">0</Property>
				<Property Name="Source[2].itemID" Type="Ref">/My Computer/LibZMQ.lvlib/libzmq-v143-mt-4_3_6.dll</Property>
				<Property Name="Source[2].sourceInclusion" Type="Str">Include</Property>
				<Property Name="SourceCount" Type="Int">3</Property>
				<Property Name="TgtF_companyName" Type="Str">Newcastle University</Property>
				<Property Name="TgtF_enableDebugging" Type="Bool">true</Property>
				<Property Name="TgtF_fileDescription" Type="Str">Zero Launched App</Property>
				<Property Name="TgtF_internalName" Type="Str">Zero Launched App</Property>
				<Property Name="TgtF_legalCopyright" Type="Str">Copyright © 2026 Newcastle University</Property>
				<Property Name="TgtF_productName" Type="Str">Zero Launched App</Property>
				<Property Name="TgtF_targetfileGUID" Type="Str">{6474F3C1-3750-47E5-96E1-1BDCD60C2FA6}</Property>
				<Property Name="TgtF_targetfileName" Type="Str">Zero Launched App.exe</Property>
				<Property Name="TgtF_versionIndependent" Type="Bool">true</Property>
			</Item>
		</Item>
	</Item>
</Project>
