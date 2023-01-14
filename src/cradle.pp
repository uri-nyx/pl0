
program Cradle;


{ Constant Declarations }

const TAB = ^I;
const CR = ^J; {LN is the way}
const A = 'a0';
const B = 'a1';
const AR = 't1';
const SP = 'sp'; { TODO: maybe leave stack pointer to system stacks}

const TOKEN_CONST = 'const';

const BANK = '#bankdef var\n{\n#addr 0x2000\n#size 0x1000\n#outp 8 * 0x2000\n}';


{ Variable Declarations }

var Look: char;              { Lookahead Character }
                              

{ Read New Character From Input Stream }

procedure GetChar;
begin
   Read(Look);
end;


{ Report an Error }

procedure Error(s: string);
begin
   WriteLn;
   WriteLn(^G, 'Error: ', s, '.');
end;



{ Report Error and Halt }

procedure Abort(s: string);
begin
   Error(s);
   Halt;
end;



{ Report What Was Expected }

procedure Expected(s: string);
begin
   Abort(s + ' Expected');
end;


{ Recognize an Alpha Character }

function IsAlpha(c: char): boolean;
begin
   IsAlpha := upcase(c) in ['A'..'Z'];
end;
                              



{ Recognize a Decimal Digit }

function IsDigit(c: char): boolean;
begin
   IsDigit := c in ['0'..'9'];
end;


{ Recognize an Alphanumeric }

function IsAlNum(c: char): boolean;
begin
   IsAlNum := IsAlpha(c) or IsDigit(c);
end;


{ Recognize White Space }

function IsWhite(c: char): boolean;
begin
   IsWhite := c in [' ', TAB];
end;


{ Skip Over Leading White Space }

procedure SkipWhite;
begin
   while IsWhite(Look) do
      GetChar;
end;


{ Match a Specific Input Character }

procedure Match(x: char);
begin
   if Look <> x then Expected('''' + x + '''')
   else begin
      GetChar;
      SkipWhite;
   end;
end;

procedure MatchString(s: string);
var Len: integer;
var i: integer;
begin
   Len := length(s);

   for i := 0 to Len do
      if Look <> s[i] then Expected('''' + s + '''')
      else begin
         GetChar;
         SkipWhite;
      end;
end;



{ Get an Identifier }

function GetName: string;
var Token: string;
begin
   Token := '';
   if not IsAlpha(Look) then Expected('Name');
   while IsAlnum(Look) do begin
      Token := Token + Look;
      GetChar;
   end;
   GetName := Token;
   SkipWhite;
end;



{ Get a Number }

function GetNum: string;
var Value: string;
begin
   Value := '';
   if not IsDigit(Look) then Expected('Integer');
   while IsDigit(Look) do begin
      Value := Value + Look;
      GetChar;
   end;
   GetNum := Value;
   SkipWhite;
end;





{ Output a String with Tab }

procedure Emit(s: string);
begin
   Write(TAB, s);
end;





{ Output a String with Tab and CRLF }

procedure EmitLn(s: string);
begin
   Emit(s);
   WriteLn;
end;


{ Initialize }

procedure Init;
begin
   GetChar;
   SkipWhite;
end;


{ Pop  to register }

procedure Pop(reg: string);
begin
   EmitLn('pop ' + reg + ', ' + SP);
end;


{ Push register }

procedure Push(reg: string);
begin
      EmitLn('push ' + reg + ', ' + SP);
end;



{ Recognize an Addop }

function IsAddop(c: char): boolean;
begin
   IsAddop := c in ['+', '-'];
end;



{ Parse and Translate an Identifier }

procedure Ident;
var Name: string;
begin
   Name := GetName;
   if Look = '(' then begin
      Match('(');
      Match(')');
      EmitLn('jal ra, ' + Name); 
      {EmitLn('call ' + Name); <- Both are PIC I think, but call can address the whole space}
      end
   else
      EmitLn('llw ' + A + ', ' + Name) { TODO: maybe prepend label with 'var_' and make it case sensitive}
end;


{ Parse and Translate a Math Factor }

procedure Expression; Forward;
procedure Factor;
begin
   if Look = '(' then begin
      Match('(');
      Expression;
      Match(')');
      end
   else if IsAlpha(Look) then
      Ident
   else
      EmitLn('li ' + A + ', ' + GetNum);
end;


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


{ Parse and Translate a Math Term }

procedure Term;
begin
   Factor;
   while Look in ['*', '/'] do begin
      Push(A);
      case Look of
       '*': Multiply;
       '/': Divide;
      end;
   end;
end;


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


{ Parse and Translate an Assignment Statement }

procedure Assignment;
var Name: string;
begin
   Name := GetName;
   Match('=');
   Expression;
   EmitLn('ssw ' + A + ', ' + Name + ', ' + AR)
end;


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
      end;
   end;
end;



procedure Constant;
var Name: string;
var Value: string;
begin
   Name := GetName;
   Match('=');
   Value := GetNum;
   EmitLn(Name + ' = ' + Value);

   if Look = ',' then begin
      Match(',');
      while Look <> ';' do
         Constant;
      end
   else if Look = ';' then
      Match(';');
end;


procedure Variable;
var Name: string;
begin
   Name := GetName;
   EmitLn('#bank var');
   EmitLn(Name + ': ');
   EmitLn('#res 4');

   if Look = ',' then begin
      Match(',');
      while Look <> ';' do
         Variable;
      end
   else if Look = ';' then
      Match(';');
end;

procedure Proc;
begin
   
end;

procedure Block;
var Keyword: string;
begin
   Keyword := GetName;
   if Keyword = 'const' then
      Constant
   else if Keyword = 'var' then
      Variable
   else if Keyword = 'procedure' then
      Proc
   else
      Expected('Block');
end;

{ Main Program }

begin
   Init;
   Block;
   if Look <> CR then Expected('Newline');
end.
