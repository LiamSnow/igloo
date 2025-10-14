// this is an example implementation of the
// mushroom light card in Home Assistant

// [input]
// target = QueryTarget

// [queries]
// switches = target.get_all(Switch).with(Light)
// dimmers = target.get_all(Dimmer).with(Light)
// colors = target.get_all(Color).with(Light)
// color_temps = target.get_all(ColorTemp).with(Light)

// [body]
<Card>
    <VStack>
		<HStack>
			<Toggle
				icon="mdi:light_group"
				value={switches.average()}
				set={switches}
			/>

			<VStack>
				<Text>
					{target.name()}
				</Text>
				<Text>
					{dimmer.average()}%
				</Text>
			</VStack>
		</HStack>

		<Tabs hide-selected-tabs="true" tab-position="Right">
			<Tab icon="mdi:brightness">
				<Slider
					value={dimmers.average()}
					set={dimmers}
				/>
			</Tab>
			<Tab icon="mdi:thermometer">
				<ColorTemperaturePicker
					value={color_temps.average()}
					set={color_temps}
					variant="Slider"
				/>
			</Tab>
			<Tab icon="mdi:color"> 
				<ColorPicker
					value={colors.average()}
					set={colors}
					variant="Slider"
				/>
			</Tab>
		</Tabs>

		{/* this is just for example purposes */}

		<Show when={dimmer > 0.0} fallback={
			<Text>OFF</Text>
		}>
			<Text>ON</Text>
		</Show>

		<For each={dimmer} of={dimmers}>
			<Text>{d.name()}</Text>
		</For>
    </VStack>
</Card>



