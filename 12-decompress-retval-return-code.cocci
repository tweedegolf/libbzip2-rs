@@
@@
- retVal = 0;
+ retVal = ReturnCode::BZ_OK;
@@
@@
- retVal = 1;
+ retVal = ReturnCode::BZ_RUN_OK;
@@
@@
- retVal = 2;
+ retVal = ReturnCode::BZ_FLUSH_OK;
@@
@@
- retVal = 3;
+ retVal = ReturnCode::BZ_FINISH_OK;
@@
@@
- retVal = 4;
+ retVal = ReturnCode::BZ_STREAM_END;
@@
@@
- retVal = -1;
+ retVal = ReturnCode::BZ_SEQUENCE_ERROR;
@@
@@
- retVal = -2;
+ retVal = ReturnCode::BZ_PARAM_ERROR;
@@
@@
- retVal = -3;
+ retVal = ReturnCode::BZ_MEM_ERROR;
@@
@@
- retVal = -4;
+ retVal = ReturnCode::BZ_DATA_ERROR;
@@
@@
- retVal = -5;
+ retVal = ReturnCode::BZ_DATA_ERROR_MAGIC;
@@
@@
- retVal = -7;
+ retVal = ReturnCode::BZ_UNEXPECTED_EOF;
@@
@@
- retVal = -8;
+ retVal = ReturnCode::BZ_OUTBUFF_FULL;
@@
@@
- retVal = -9;
+ retVal = ReturnCode::BZ_CONFIG_ERROR;
