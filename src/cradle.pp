{--------------------------------------------------------------}
program Cradle;

{--------------------------------------------------------------}
{ Constant Declarations }

const TAB = ^I;
const A = 'a0';
const B = 'a1';
const SP = 'sp'; { TODO: maybe leave stack pointer to system stacks}

{--------------------------------------------------------------}
{ Variable Declarations }

var Look: char;              { Lookahead Character }
                              
{--------------------------------------------------------------}
{ Read New Character From Input Stream }

procedure GetChar;
begin
   Read(Look);
end;

{--------------------------------------------------------------}
{ Report an Error }

procedure Error(s: string);
begin
   WriteLn;
   WriteLn(^G, 'Error: ', s, '.');
end;


{--------------------------------------------------------------}
{ Report Error and Halt }

procedure Abort(s: string);
begin
   Error(s);
   Halt;
end;


{--------------------------------------------------------------}
{ Report What Was Expected }

procedure Expected(s: string);
begin
   Abort(s + ' Expected');
end;

{--------------------------------------------------------------}
{ Match a Specific Input Character }

procedure Match(x: char);
begin
   if Look = x then GetChar
   else Expected('''' + x + '''');
end;


{--------------------------------------------------------------}
{ Recognize an Alpha Character }

function IsAlpha(c: char): boolean;
begin
   IsAlpha := upcase(c) in ['A'..'Z'];
end;
                              

{--------------------------------------------------------------}

{ Recognize a Decimal Digit }

function IsDigit(c: char): boolean;
begin
   IsDigit := c in ['0'..'9'];
end;


{--------------------------------------------------------------}
{ Get an Identifier }

function GetName: char;
begin
   if not IsAlpha(Look) then Expected('Name');
   GetName := UpCase(Look);
   GetChar;
end;


{--------------------------------------------------------------}
{ Get a Number }

function GetNum: char;
begin
   if not IsDigit(Look) then Expected('Integer');
   GetNum := Look;
   GetChar;
end;


{--------------------------------------------------------------}
{ Output a String with Tab }

procedure Emit(s: string);
begin
   Write(TAB, s);
end;




{--------------------------------------------------------------}
{ Output a String with Tab and CRLF }

procedure EmitLn(s: string);
begin
   Emit(s);
   WriteLn;
end;

{--------------------------------------------------------------}
{ Initialize }

procedure Init;
begin
   GetChar;
end;

{---------------------------------------------------------------}
{ Pop  to register }

procedure Pop(reg: string);
begin
   EmitLn('pop ' + reg + ', ' + SP);
end;

{---------------------------------------------------------------}
{ Push register }

procedure Push(reg: string);
begin
      EmitLn('push ' + reg + ', ' + SP);
end;


{--------------------------------------------------------------}
{ Recognize an Addop }

function IsAddop(c: char): boolean;
begin
   IsAddop := c in ['+', '-'];
end;
{--------------------------------------------------------------}


{---------------------------------------------------------------}
{ Parse and Translate a Math Factor }

procedure Expression; Forward;
procedure Factor;
begin
   if Look = '(' then begin
      Match('(');
      Expression;
      Match(')');
      end
   else
      EmitLn('li a0, ' + GetNum);
end;

{--------------------------------------------------------------}
{ Recognize and Translate a Multiply }

procedure Multiply;
begin
   Match('*');
   Factor;
   Pop(B);
   EmitLn('mul zero, ' + A + ', ' + A + ', ' + B); { TODO: Does not handle overflow}
end;

{-------------------------------------------------------------}
{ Recognize and Translate a Divide }

procedure Divide;
begin
   Match('/');
   Factor;
   Pop(B);
   EmitLn('idiv ' + A + ', ' + 'zero, ' + B + ', ' + A);
end;

{---------------------------------------------------------------}
{ Parse and Translate a Math Term }

procedure Term;
begin
   Factor;
   while Look in ['*', '/'] do begin
      Push(A);
      case Look of
       '*': Multiply;
       '/': Divide;
      else Expected('MulOp');
      end;
   end;
end;

{--------------------------------------------------------------}
{ Recognize and Translate an Add }

procedure Add;
begin
   Match('+');
   Term;
   Pop(B);
   EmitLn('add ' + A + ', ' + A + ', ' + B);
end;


{-------------------------------------------------------------}
{ Recognize and Translate a Subtract }

procedure Subtract;
begin
   Match('-');
   Term;
   Pop(B);
   EmitLn('sub '+ A + ', ' + B + ', ' + A);
end;
{-------------------------------------------------------------}

procedure Expression;
begin
   if IsAddop(Look) then
      EmitLn('mv ' + A + ', zero')
   else
      Term;
   while IsAddop(Look) do begin
      Push(A);
      case Look of
         '+': Add;
         '-': Subtract;
         else Expected('AddOp');
      end;
   end;
end; 
{---------------------------------------------------------------}



{--------------------------------------------------------------}
{ Main Program }

begin
   Init;
   Expression;
end.
{--------------------------------------------------------------}