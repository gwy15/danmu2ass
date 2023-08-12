<template>
  <div>
    <v-row>
      <v-col cols="6" md="3">
        <my-number-field v-model="config.width" label="屏幕宽度" suffix="px" />
      </v-col>
      <v-col cols="6" md="3">
        <my-number-field v-model="config.height" label="屏幕高度" suffix="px" />
      </v-col>
      <v-col cols="6" md="3">
        <my-percentage-field
          v-model="config.float_percentage"
          label="弹幕显示区域"
        />
      </v-col>
    </v-row>
    <v-row>
      <v-col cols="6" md="3">
        <my-number-field
          v-model="config.font_size"
          suffix="px"
          label="弹幕字体大小"
        />
      </v-col>
      <v-col cols="6" md="3">
        <my-number-field v-model="config.width_ratio" label="弹幕宽度乘子" />
      </v-col>
      <v-col cols="6" md="3">
        <my-number-field
          v-model="config.horizontal_gap"
          suffix="px"
          label="弹幕之间最小水平间距"
        />
      </v-col>
      <v-col cols="6" md="3">
        <my-number-field v-model="config.lane_size" suffix="px" label="行高" />
      </v-col>
    </v-row>
    <v-row>
      <v-col cols="6" md="3">
        <my-text-field v-model="config.font" label="字体" />
      </v-col>
      <v-col cols="6" md="3">
        <v-checkbox
          dense
          hide-details
          v-model="config.bold"
          label="加粗"
        ></v-checkbox>
      </v-col>
      <v-col cols="6" md="3">
        <my-number-field
          v-model="config.outline"
          suffix="px"
          label="描边宽度"
        />
      </v-col>
      <v-col cols="6" md="3">
        <my-percentage-field v-model="config.alpha" label="弹幕不透明度" />
      </v-col>
    </v-row>
    <v-row>
      <v-col cols="6" md="3">
        <my-number-field
          v-model="config.duration"
          label="弹幕在屏幕上的持续时间"
          suffix="秒"
        />
      </v-col>
      <v-col cols="6" md="3">
        <my-number-field
          v-model="config.time_offset"
          label="弹幕延后"
          suffix="秒"
        />
      </v-col>
      <slot></slot>
    </v-row>
  </div>
</template>
  
<script>
import MyPercentageField from "./MyPercentageField.vue";
import MyNumberField from "./MyNumberField.vue";

export default {
  name: "ConfigEditor",
  components: {
    MyPercentageField,
    MyNumberField,
  },
  props: {
    value: {
      type: Object,
      required: true,
    },
  },
  computed: {
    config: {
      get() {
        return this.value;
      },
      set(newValue) {
        this.$emit("input", newValue);
      },
    },
  },
};
</script>