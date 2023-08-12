import Vue from 'vue';
import Vuetify from 'vuetify/lib/framework';
import { VTextField } from 'vuetify/lib';

Vue.use(Vuetify);

export default new Vuetify({});

Vue.component('MyTextField', {
  extends: VTextField,
  props: {
    outlined: {
      type: Boolean,
      default: true
    },
    dense: {
      type: Boolean,
      default: true,
    },
    hideDetails: {
      type: Boolean,
      default: true,
    }
  }
})
