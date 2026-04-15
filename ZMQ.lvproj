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
			<Item Name="ReqRep.vi" Type="VI" URL="../src/Examples/ReqRep.vi"/>
			<Item Name="PubSub.vi" Type="VI" URL="../src/Examples/PubSub.vi"/>
			<Item Name="PubSubSync.vi" Type="VI" URL="../src/Examples/PubSubSync.vi"/>
		</Item>
		<Item Name="Example.SubChat.vi" Type="VI" URL="../Example.SubChat.vi"/>
		<Item Name="Example.vi" Type="VI" URL="../Example.vi"/>
		<Item Name="Linux.Test.vi" Type="VI" URL="../Linux.Test.vi"/>
		<Item Name="ExampleClient.vi" Type="VI" URL="../src/ExampleClient.vi"/>
		<Item Name="ZeroMQ.lvlib" Type="Library" URL="../src/zeromq/ZeroMQ.lvlib"/>
		<Item Name="LibZMQ.lvlib" Type="Library" URL="../libzmq-v143-mt-4_3_6/LibZMQ.lvlib"/>
		<Item Name="Dependencies" Type="Dependencies"/>
		<Item Name="Build Specifications" Type="Build"/>
	</Item>
</Project>
